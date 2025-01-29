// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, clippy::match_same_arms)]

use serde::{ser::SerializeMap, Serialize, Serializer};

use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

impl Serialize for Program<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Program")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("sourceType", &self.source_type)?;
        map.serialize_entry("hashbang", &self.hashbang)?;
        map.serialize_entry("directives", &self.directives)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for Expression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Expression::BooleanLiteral(it) => Serialize::serialize(it, serializer),
            Expression::NullLiteral(it) => Serialize::serialize(it, serializer),
            Expression::NumericLiteral(it) => Serialize::serialize(it, serializer),
            Expression::BigIntLiteral(it) => Serialize::serialize(it, serializer),
            Expression::RegExpLiteral(it) => Serialize::serialize(it, serializer),
            Expression::StringLiteral(it) => Serialize::serialize(it, serializer),
            Expression::TemplateLiteral(it) => Serialize::serialize(it, serializer),
            Expression::Identifier(it) => Serialize::serialize(it, serializer),
            Expression::MetaProperty(it) => Serialize::serialize(it, serializer),
            Expression::Super(it) => Serialize::serialize(it, serializer),
            Expression::ArrayExpression(it) => Serialize::serialize(it, serializer),
            Expression::ArrowFunctionExpression(it) => Serialize::serialize(it, serializer),
            Expression::AssignmentExpression(it) => Serialize::serialize(it, serializer),
            Expression::AwaitExpression(it) => Serialize::serialize(it, serializer),
            Expression::BinaryExpression(it) => Serialize::serialize(it, serializer),
            Expression::CallExpression(it) => Serialize::serialize(it, serializer),
            Expression::ChainExpression(it) => Serialize::serialize(it, serializer),
            Expression::ClassExpression(it) => Serialize::serialize(it, serializer),
            Expression::ConditionalExpression(it) => Serialize::serialize(it, serializer),
            Expression::FunctionExpression(it) => Serialize::serialize(it, serializer),
            Expression::ImportExpression(it) => Serialize::serialize(it, serializer),
            Expression::LogicalExpression(it) => Serialize::serialize(it, serializer),
            Expression::NewExpression(it) => Serialize::serialize(it, serializer),
            Expression::ObjectExpression(it) => Serialize::serialize(it, serializer),
            Expression::ParenthesizedExpression(it) => Serialize::serialize(it, serializer),
            Expression::SequenceExpression(it) => Serialize::serialize(it, serializer),
            Expression::TaggedTemplateExpression(it) => Serialize::serialize(it, serializer),
            Expression::ThisExpression(it) => Serialize::serialize(it, serializer),
            Expression::UnaryExpression(it) => Serialize::serialize(it, serializer),
            Expression::UpdateExpression(it) => Serialize::serialize(it, serializer),
            Expression::YieldExpression(it) => Serialize::serialize(it, serializer),
            Expression::PrivateInExpression(it) => Serialize::serialize(it, serializer),
            Expression::JSXElement(it) => Serialize::serialize(it, serializer),
            Expression::JSXFragment(it) => Serialize::serialize(it, serializer),
            Expression::TSAsExpression(it) => Serialize::serialize(it, serializer),
            Expression::TSSatisfiesExpression(it) => Serialize::serialize(it, serializer),
            Expression::TSTypeAssertion(it) => Serialize::serialize(it, serializer),
            Expression::TSNonNullExpression(it) => Serialize::serialize(it, serializer),
            Expression::TSInstantiationExpression(it) => Serialize::serialize(it, serializer),
            Expression::ComputedMemberExpression(it) => Serialize::serialize(it, serializer),
            Expression::StaticMemberExpression(it) => Serialize::serialize(it, serializer),
            Expression::PrivateFieldExpression(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for IdentifierName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Identifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

impl Serialize for IdentifierReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Identifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

impl Serialize for BindingIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Identifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

impl Serialize for LabelIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Identifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

impl Serialize for ThisExpression {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ThisExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for ArrayExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ArrayExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("elements", &self.elements)?;
        map.end()
    }
}

impl Serialize for ArrayExpressionElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ArrayExpressionElement::SpreadElement(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::Elision(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::BooleanLiteral(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::NullLiteral(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::NumericLiteral(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::BigIntLiteral(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::RegExpLiteral(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::StringLiteral(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::TemplateLiteral(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::Identifier(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::MetaProperty(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::Super(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::ArrayExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::ArrowFunctionExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ArrayExpressionElement::AssignmentExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ArrayExpressionElement::AwaitExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::BinaryExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::CallExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::ChainExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::ClassExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::ConditionalExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ArrayExpressionElement::FunctionExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::ImportExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::LogicalExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::NewExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::ObjectExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::ParenthesizedExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ArrayExpressionElement::SequenceExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::TaggedTemplateExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ArrayExpressionElement::ThisExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::UnaryExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::UpdateExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::YieldExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::PrivateInExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::JSXElement(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::JSXFragment(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::TSAsExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::TSSatisfiesExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ArrayExpressionElement::TSTypeAssertion(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::TSNonNullExpression(it) => Serialize::serialize(it, serializer),
            ArrayExpressionElement::TSInstantiationExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ArrayExpressionElement::ComputedMemberExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ArrayExpressionElement::StaticMemberExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ArrayExpressionElement::PrivateFieldExpression(it) => {
                Serialize::serialize(it, serializer)
            }
        }
    }
}

impl Serialize for ObjectExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ObjectExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("properties", &self.properties)?;
        map.end()
    }
}

impl Serialize for ObjectPropertyKind<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ObjectPropertyKind::ObjectProperty(it) => Serialize::serialize(it, serializer),
            ObjectPropertyKind::SpreadProperty(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for ObjectProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ObjectProperty")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
            PropertyKey::StaticIdentifier(it) => Serialize::serialize(it, serializer),
            PropertyKey::PrivateIdentifier(it) => Serialize::serialize(it, serializer),
            PropertyKey::BooleanLiteral(it) => Serialize::serialize(it, serializer),
            PropertyKey::NullLiteral(it) => Serialize::serialize(it, serializer),
            PropertyKey::NumericLiteral(it) => Serialize::serialize(it, serializer),
            PropertyKey::BigIntLiteral(it) => Serialize::serialize(it, serializer),
            PropertyKey::RegExpLiteral(it) => Serialize::serialize(it, serializer),
            PropertyKey::StringLiteral(it) => Serialize::serialize(it, serializer),
            PropertyKey::TemplateLiteral(it) => Serialize::serialize(it, serializer),
            PropertyKey::Identifier(it) => Serialize::serialize(it, serializer),
            PropertyKey::MetaProperty(it) => Serialize::serialize(it, serializer),
            PropertyKey::Super(it) => Serialize::serialize(it, serializer),
            PropertyKey::ArrayExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::ArrowFunctionExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::AssignmentExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::AwaitExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::BinaryExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::CallExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::ChainExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::ClassExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::ConditionalExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::FunctionExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::ImportExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::LogicalExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::NewExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::ObjectExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::ParenthesizedExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::SequenceExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::TaggedTemplateExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::ThisExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::UnaryExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::UpdateExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::YieldExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::PrivateInExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::JSXElement(it) => Serialize::serialize(it, serializer),
            PropertyKey::JSXFragment(it) => Serialize::serialize(it, serializer),
            PropertyKey::TSAsExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::TSSatisfiesExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::TSTypeAssertion(it) => Serialize::serialize(it, serializer),
            PropertyKey::TSNonNullExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::TSInstantiationExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::ComputedMemberExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::StaticMemberExpression(it) => Serialize::serialize(it, serializer),
            PropertyKey::PrivateFieldExpression(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for PropertyKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("quasis", &self.quasis)?;
        map.serialize_entry("expressions", &self.expressions)?;
        map.end()
    }
}

impl Serialize for TaggedTemplateExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TaggedTemplateExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
            MemberExpression::ComputedMemberExpression(it) => Serialize::serialize(it, serializer),
            MemberExpression::StaticMemberExpression(it) => Serialize::serialize(it, serializer),
            MemberExpression::PrivateFieldExpression(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for ComputedMemberExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ComputedMemberExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("object", &self.object)?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("optional", &self.optional)?;
        map.end()
    }
}

impl Serialize for StaticMemberExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "StaticMemberExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("object", &self.object)?;
        map.serialize_entry("property", &self.property)?;
        map.serialize_entry("optional", &self.optional)?;
        map.end()
    }
}

impl Serialize for PrivateFieldExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "PrivateFieldExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("object", &self.object)?;
        map.serialize_entry("field", &self.field)?;
        map.serialize_entry("optional", &self.optional)?;
        map.end()
    }
}

impl Serialize for CallExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CallExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("meta", &self.meta)?;
        map.serialize_entry("property", &self.property)?;
        map.end()
    }
}

impl Serialize for SpreadElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "SpreadElement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for Argument<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Argument::SpreadElement(it) => Serialize::serialize(it, serializer),
            Argument::BooleanLiteral(it) => Serialize::serialize(it, serializer),
            Argument::NullLiteral(it) => Serialize::serialize(it, serializer),
            Argument::NumericLiteral(it) => Serialize::serialize(it, serializer),
            Argument::BigIntLiteral(it) => Serialize::serialize(it, serializer),
            Argument::RegExpLiteral(it) => Serialize::serialize(it, serializer),
            Argument::StringLiteral(it) => Serialize::serialize(it, serializer),
            Argument::TemplateLiteral(it) => Serialize::serialize(it, serializer),
            Argument::Identifier(it) => Serialize::serialize(it, serializer),
            Argument::MetaProperty(it) => Serialize::serialize(it, serializer),
            Argument::Super(it) => Serialize::serialize(it, serializer),
            Argument::ArrayExpression(it) => Serialize::serialize(it, serializer),
            Argument::ArrowFunctionExpression(it) => Serialize::serialize(it, serializer),
            Argument::AssignmentExpression(it) => Serialize::serialize(it, serializer),
            Argument::AwaitExpression(it) => Serialize::serialize(it, serializer),
            Argument::BinaryExpression(it) => Serialize::serialize(it, serializer),
            Argument::CallExpression(it) => Serialize::serialize(it, serializer),
            Argument::ChainExpression(it) => Serialize::serialize(it, serializer),
            Argument::ClassExpression(it) => Serialize::serialize(it, serializer),
            Argument::ConditionalExpression(it) => Serialize::serialize(it, serializer),
            Argument::FunctionExpression(it) => Serialize::serialize(it, serializer),
            Argument::ImportExpression(it) => Serialize::serialize(it, serializer),
            Argument::LogicalExpression(it) => Serialize::serialize(it, serializer),
            Argument::NewExpression(it) => Serialize::serialize(it, serializer),
            Argument::ObjectExpression(it) => Serialize::serialize(it, serializer),
            Argument::ParenthesizedExpression(it) => Serialize::serialize(it, serializer),
            Argument::SequenceExpression(it) => Serialize::serialize(it, serializer),
            Argument::TaggedTemplateExpression(it) => Serialize::serialize(it, serializer),
            Argument::ThisExpression(it) => Serialize::serialize(it, serializer),
            Argument::UnaryExpression(it) => Serialize::serialize(it, serializer),
            Argument::UpdateExpression(it) => Serialize::serialize(it, serializer),
            Argument::YieldExpression(it) => Serialize::serialize(it, serializer),
            Argument::PrivateInExpression(it) => Serialize::serialize(it, serializer),
            Argument::JSXElement(it) => Serialize::serialize(it, serializer),
            Argument::JSXFragment(it) => Serialize::serialize(it, serializer),
            Argument::TSAsExpression(it) => Serialize::serialize(it, serializer),
            Argument::TSSatisfiesExpression(it) => Serialize::serialize(it, serializer),
            Argument::TSTypeAssertion(it) => Serialize::serialize(it, serializer),
            Argument::TSNonNullExpression(it) => Serialize::serialize(it, serializer),
            Argument::TSInstantiationExpression(it) => Serialize::serialize(it, serializer),
            Argument::ComputedMemberExpression(it) => Serialize::serialize(it, serializer),
            Argument::StaticMemberExpression(it) => Serialize::serialize(it, serializer),
            Argument::PrivateFieldExpression(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for UpdateExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "UpdateExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("operator", &self.operator)?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for BinaryExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BinaryExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("operator", &self.operator)?;
        map.serialize_entry("left", &self.left)?;
        map.serialize_entry("right", &self.right)?;
        map.end()
    }
}

impl Serialize for AssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            AssignmentTarget::AssignmentTargetIdentifier(it) => {
                Serialize::serialize(it, serializer)
            }
            AssignmentTarget::TSAsExpression(it) => Serialize::serialize(it, serializer),
            AssignmentTarget::TSSatisfiesExpression(it) => Serialize::serialize(it, serializer),
            AssignmentTarget::TSNonNullExpression(it) => Serialize::serialize(it, serializer),
            AssignmentTarget::TSTypeAssertion(it) => Serialize::serialize(it, serializer),
            AssignmentTarget::TSInstantiationExpression(it) => Serialize::serialize(it, serializer),
            AssignmentTarget::ComputedMemberExpression(it) => Serialize::serialize(it, serializer),
            AssignmentTarget::StaticMemberExpression(it) => Serialize::serialize(it, serializer),
            AssignmentTarget::PrivateFieldExpression(it) => Serialize::serialize(it, serializer),
            AssignmentTarget::ArrayAssignmentTarget(it) => Serialize::serialize(it, serializer),
            AssignmentTarget::ObjectAssignmentTarget(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for SimpleAssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(it) => {
                Serialize::serialize(it, serializer)
            }
            SimpleAssignmentTarget::TSAsExpression(it) => Serialize::serialize(it, serializer),
            SimpleAssignmentTarget::TSSatisfiesExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            SimpleAssignmentTarget::TSNonNullExpression(it) => Serialize::serialize(it, serializer),
            SimpleAssignmentTarget::TSTypeAssertion(it) => Serialize::serialize(it, serializer),
            SimpleAssignmentTarget::TSInstantiationExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            SimpleAssignmentTarget::ComputedMemberExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            SimpleAssignmentTarget::StaticMemberExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            SimpleAssignmentTarget::PrivateFieldExpression(it) => {
                Serialize::serialize(it, serializer)
            }
        }
    }
}

impl Serialize for AssignmentTargetPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            AssignmentTargetPattern::ArrayAssignmentTarget(it) => {
                Serialize::serialize(it, serializer)
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(it) => {
                Serialize::serialize(it, serializer)
            }
        }
    }
}

impl Serialize for ArrayAssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ArrayAssignmentTarget")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry(
            "elements",
            &oxc_estree::ser::AppendTo { array: &self.elements, after: &self.rest },
        )?;
        map.end()
    }
}

impl Serialize for ObjectAssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ObjectAssignmentTarget")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry(
            "properties",
            &oxc_estree::ser::AppendTo { array: &self.properties, after: &self.rest },
        )?;
        map.end()
    }
}

impl Serialize for AssignmentTargetRest<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "RestElement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("argument", &self.target)?;
        map.end()
    }
}

impl Serialize for AssignmentTargetMaybeDefault<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(it) => {
                Serialize::serialize(it, serializer)
            }
            AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(it) => {
                Serialize::serialize(it, serializer)
            }
            AssignmentTargetMaybeDefault::TSAsExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            AssignmentTargetMaybeDefault::TSSatisfiesExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            AssignmentTargetMaybeDefault::TSNonNullExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            AssignmentTargetMaybeDefault::TSTypeAssertion(it) => {
                Serialize::serialize(it, serializer)
            }
            AssignmentTargetMaybeDefault::TSInstantiationExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            AssignmentTargetMaybeDefault::ComputedMemberExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            AssignmentTargetMaybeDefault::StaticMemberExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            AssignmentTargetMaybeDefault::PrivateFieldExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            AssignmentTargetMaybeDefault::ArrayAssignmentTarget(it) => {
                Serialize::serialize(it, serializer)
            }
            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(it) => {
                Serialize::serialize(it, serializer)
            }
        }
    }
}

impl Serialize for AssignmentTargetWithDefault<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AssignmentTargetWithDefault")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("binding", &self.binding)?;
        map.serialize_entry("init", &self.init)?;
        map.end()
    }
}

impl Serialize for AssignmentTargetProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(it) => {
                Serialize::serialize(it, serializer)
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(it) => {
                Serialize::serialize(it, serializer)
            }
        }
    }
}

impl Serialize for AssignmentTargetPropertyIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AssignmentTargetPropertyIdentifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("binding", &self.binding)?;
        map.serialize_entry("init", &self.init)?;
        map.end()
    }
}

impl Serialize for AssignmentTargetPropertyProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AssignmentTargetPropertyProperty")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expressions", &self.expressions)?;
        map.end()
    }
}

impl Serialize for Super {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Super")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for AwaitExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AwaitExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for ChainExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ChainExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for ChainElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ChainElement::CallExpression(it) => Serialize::serialize(it, serializer),
            ChainElement::TSNonNullExpression(it) => Serialize::serialize(it, serializer),
            ChainElement::ComputedMemberExpression(it) => Serialize::serialize(it, serializer),
            ChainElement::StaticMemberExpression(it) => Serialize::serialize(it, serializer),
            ChainElement::PrivateFieldExpression(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for ParenthesizedExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ParenthesizedExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for Statement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Statement::BlockStatement(it) => Serialize::serialize(it, serializer),
            Statement::BreakStatement(it) => Serialize::serialize(it, serializer),
            Statement::ContinueStatement(it) => Serialize::serialize(it, serializer),
            Statement::DebuggerStatement(it) => Serialize::serialize(it, serializer),
            Statement::DoWhileStatement(it) => Serialize::serialize(it, serializer),
            Statement::EmptyStatement(it) => Serialize::serialize(it, serializer),
            Statement::ExpressionStatement(it) => Serialize::serialize(it, serializer),
            Statement::ForInStatement(it) => Serialize::serialize(it, serializer),
            Statement::ForOfStatement(it) => Serialize::serialize(it, serializer),
            Statement::ForStatement(it) => Serialize::serialize(it, serializer),
            Statement::IfStatement(it) => Serialize::serialize(it, serializer),
            Statement::LabeledStatement(it) => Serialize::serialize(it, serializer),
            Statement::ReturnStatement(it) => Serialize::serialize(it, serializer),
            Statement::SwitchStatement(it) => Serialize::serialize(it, serializer),
            Statement::ThrowStatement(it) => Serialize::serialize(it, serializer),
            Statement::TryStatement(it) => Serialize::serialize(it, serializer),
            Statement::WhileStatement(it) => Serialize::serialize(it, serializer),
            Statement::WithStatement(it) => Serialize::serialize(it, serializer),
            Statement::VariableDeclaration(it) => Serialize::serialize(it, serializer),
            Statement::FunctionDeclaration(it) => Serialize::serialize(it, serializer),
            Statement::ClassDeclaration(it) => Serialize::serialize(it, serializer),
            Statement::TSTypeAliasDeclaration(it) => Serialize::serialize(it, serializer),
            Statement::TSInterfaceDeclaration(it) => Serialize::serialize(it, serializer),
            Statement::TSEnumDeclaration(it) => Serialize::serialize(it, serializer),
            Statement::TSModuleDeclaration(it) => Serialize::serialize(it, serializer),
            Statement::TSImportEqualsDeclaration(it) => Serialize::serialize(it, serializer),
            Statement::ImportDeclaration(it) => Serialize::serialize(it, serializer),
            Statement::ExportAllDeclaration(it) => Serialize::serialize(it, serializer),
            Statement::ExportDefaultDeclaration(it) => Serialize::serialize(it, serializer),
            Statement::ExportNamedDeclaration(it) => Serialize::serialize(it, serializer),
            Statement::TSExportAssignment(it) => Serialize::serialize(it, serializer),
            Statement::TSNamespaceExportDeclaration(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for Directive<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Directive")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("directive", &self.directive)?;
        map.end()
    }
}

impl Serialize for Hashbang<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Hashbang")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for BlockStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BlockStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for Declaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Declaration::VariableDeclaration(it) => Serialize::serialize(it, serializer),
            Declaration::FunctionDeclaration(it) => Serialize::serialize(it, serializer),
            Declaration::ClassDeclaration(it) => Serialize::serialize(it, serializer),
            Declaration::TSTypeAliasDeclaration(it) => Serialize::serialize(it, serializer),
            Declaration::TSInterfaceDeclaration(it) => Serialize::serialize(it, serializer),
            Declaration::TSEnumDeclaration(it) => Serialize::serialize(it, serializer),
            Declaration::TSModuleDeclaration(it) => Serialize::serialize(it, serializer),
            Declaration::TSImportEqualsDeclaration(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for VariableDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "VariableDeclaration")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("declarations", &self.declarations)?;
        map.serialize_entry("declare", &self.declare)?;
        map.end()
    }
}

impl Serialize for VariableDeclarationKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for ExpressionStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ExpressionStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for IfStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "IfStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.serialize_entry("test", &self.test)?;
        map.end()
    }
}

impl Serialize for WhileStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "WhileStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("test", &self.test)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for ForStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ForStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
            ForStatementInit::VariableDeclaration(it) => Serialize::serialize(it, serializer),
            ForStatementInit::BooleanLiteral(it) => Serialize::serialize(it, serializer),
            ForStatementInit::NullLiteral(it) => Serialize::serialize(it, serializer),
            ForStatementInit::NumericLiteral(it) => Serialize::serialize(it, serializer),
            ForStatementInit::BigIntLiteral(it) => Serialize::serialize(it, serializer),
            ForStatementInit::RegExpLiteral(it) => Serialize::serialize(it, serializer),
            ForStatementInit::StringLiteral(it) => Serialize::serialize(it, serializer),
            ForStatementInit::TemplateLiteral(it) => Serialize::serialize(it, serializer),
            ForStatementInit::Identifier(it) => Serialize::serialize(it, serializer),
            ForStatementInit::MetaProperty(it) => Serialize::serialize(it, serializer),
            ForStatementInit::Super(it) => Serialize::serialize(it, serializer),
            ForStatementInit::ArrayExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::ArrowFunctionExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::AssignmentExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::AwaitExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::BinaryExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::CallExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::ChainExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::ClassExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::ConditionalExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::FunctionExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::ImportExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::LogicalExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::NewExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::ObjectExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::ParenthesizedExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::SequenceExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::TaggedTemplateExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::ThisExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::UnaryExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::UpdateExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::YieldExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::PrivateInExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::JSXElement(it) => Serialize::serialize(it, serializer),
            ForStatementInit::JSXFragment(it) => Serialize::serialize(it, serializer),
            ForStatementInit::TSAsExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::TSSatisfiesExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::TSTypeAssertion(it) => Serialize::serialize(it, serializer),
            ForStatementInit::TSNonNullExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::TSInstantiationExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::ComputedMemberExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::StaticMemberExpression(it) => Serialize::serialize(it, serializer),
            ForStatementInit::PrivateFieldExpression(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for ForInStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ForInStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("left", &self.left)?;
        map.serialize_entry("right", &self.right)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for ForStatementLeft<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ForStatementLeft::VariableDeclaration(it) => Serialize::serialize(it, serializer),
            ForStatementLeft::AssignmentTargetIdentifier(it) => {
                Serialize::serialize(it, serializer)
            }
            ForStatementLeft::TSAsExpression(it) => Serialize::serialize(it, serializer),
            ForStatementLeft::TSSatisfiesExpression(it) => Serialize::serialize(it, serializer),
            ForStatementLeft::TSNonNullExpression(it) => Serialize::serialize(it, serializer),
            ForStatementLeft::TSTypeAssertion(it) => Serialize::serialize(it, serializer),
            ForStatementLeft::TSInstantiationExpression(it) => Serialize::serialize(it, serializer),
            ForStatementLeft::ComputedMemberExpression(it) => Serialize::serialize(it, serializer),
            ForStatementLeft::StaticMemberExpression(it) => Serialize::serialize(it, serializer),
            ForStatementLeft::PrivateFieldExpression(it) => Serialize::serialize(it, serializer),
            ForStatementLeft::ArrayAssignmentTarget(it) => Serialize::serialize(it, serializer),
            ForStatementLeft::ObjectAssignmentTarget(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for ForOfStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ForOfStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("label", &self.label)?;
        map.end()
    }
}

impl Serialize for BreakStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BreakStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("label", &self.label)?;
        map.end()
    }
}

impl Serialize for ReturnStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ReturnStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for WithStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "WithStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("object", &self.object)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for SwitchStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "SwitchStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("discriminant", &self.discriminant)?;
        map.serialize_entry("cases", &self.cases)?;
        map.end()
    }
}

impl Serialize for SwitchCase<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "SwitchCase")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("test", &self.test)?;
        map.serialize_entry("consequent", &self.consequent)?;
        map.end()
    }
}

impl Serialize for LabeledStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "LabeledStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("label", &self.label)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for ThrowStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ThrowStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for TryStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TryStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("param", &self.param)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for CatchParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CatchParameter")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("pattern", &self.pattern)?;
        map.end()
    }
}

impl Serialize for DebuggerStatement {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "DebuggerStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for BindingPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        self.kind.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("optional", &self.optional)?;
        map.end()
    }
}

impl Serialize for BindingPatternKind<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            BindingPatternKind::BindingIdentifier(it) => Serialize::serialize(it, serializer),
            BindingPatternKind::ObjectPattern(it) => Serialize::serialize(it, serializer),
            BindingPatternKind::ArrayPattern(it) => Serialize::serialize(it, serializer),
            BindingPatternKind::AssignmentPattern(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for AssignmentPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AssignmentPattern")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("left", &self.left)?;
        map.serialize_entry("right", &self.right)?;
        map.end()
    }
}

impl Serialize for ObjectPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ObjectPattern")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry(
            "properties",
            &oxc_estree::ser::AppendTo { array: &self.properties, after: &self.rest },
        )?;
        map.end()
    }
}

impl Serialize for BindingProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BindingProperty")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry(
            "elements",
            &oxc_estree::ser::AppendTo { array: &self.elements, after: &self.rest },
        )?;
        map.end()
    }
}

impl Serialize for BindingRestElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "RestElement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for Function<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        match *self {
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("decorators", &self.decorators)?;
        self.pattern.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("accessibility", &self.accessibility)?;
        map.serialize_entry("readonly", &self.readonly)?;
        map.serialize_entry("override", &self.r#override)?;
        map.end()
    }
}

impl Serialize for FormalParameterKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("directives", &self.directives)?;
        map.serialize_entry("statements", &self.statements)?;
        map.end()
    }
}

impl Serialize for ArrowFunctionExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ArrowFunctionExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("delegate", &self.delegate)?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for Class<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        match *self {
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for ClassElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ClassElement::StaticBlock(it) => Serialize::serialize(it, serializer),
            ClassElement::MethodDefinition(it) => Serialize::serialize(it, serializer),
            ClassElement::PropertyDefinition(it) => Serialize::serialize(it, serializer),
            ClassElement::AccessorProperty(it) => Serialize::serialize(it, serializer),
            ClassElement::TSIndexSignature(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for MethodDefinition<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        match *self {
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        match *self {
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
        match *self {
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

impl Serialize for StaticBlock<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "StaticBlock")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for ModuleDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ModuleDeclaration::ImportDeclaration(it) => Serialize::serialize(it, serializer),
            ModuleDeclaration::ExportAllDeclaration(it) => Serialize::serialize(it, serializer),
            ModuleDeclaration::ExportDefaultDeclaration(it) => Serialize::serialize(it, serializer),
            ModuleDeclaration::ExportNamedDeclaration(it) => Serialize::serialize(it, serializer),
            ModuleDeclaration::TSExportAssignment(it) => Serialize::serialize(it, serializer),
            ModuleDeclaration::TSNamespaceExportDeclaration(it) => {
                Serialize::serialize(it, serializer)
            }
        }
    }
}

impl Serialize for AccessorPropertyType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        match *self {
            ImportPhase::Source => serializer.serialize_unit_variant("ImportPhase", 0, "source"),
            ImportPhase::Defer => serializer.serialize_unit_variant("ImportPhase", 1, "defer"),
        }
    }
}

impl Serialize for ImportDeclarationSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ImportDeclarationSpecifier::ImportSpecifier(it) => Serialize::serialize(it, serializer),
            ImportDeclarationSpecifier::ImportDefaultSpecifier(it) => {
                Serialize::serialize(it, serializer)
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(it) => {
                Serialize::serialize(it, serializer)
            }
        }
    }
}

impl Serialize for ImportSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ImportSpecifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("local", &self.local)?;
        map.end()
    }
}

impl Serialize for ImportNamespaceSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ImportNamespaceSpecifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("local", &self.local)?;
        map.end()
    }
}

impl Serialize for WithClause<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "WithClause")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("attributesKeyword", &self.attributes_keyword)?;
        map.serialize_entry("withEntries", &self.with_entries)?;
        map.end()
    }
}

impl Serialize for ImportAttribute<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ImportAttribute")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("key", &self.key)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for ImportAttributeKey<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ImportAttributeKey::Identifier(it) => Serialize::serialize(it, serializer),
            ImportAttributeKey::StringLiteral(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for ExportNamedDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ExportNamedDeclaration")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("declaration", &self.declaration)?;
        map.serialize_entry("exported", &self.exported)?;
        map.end()
    }
}

impl Serialize for ExportAllDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ExportAllDeclaration")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("local", &self.local)?;
        map.serialize_entry("exported", &self.exported)?;
        map.serialize_entry("exportKind", &self.export_kind)?;
        map.end()
    }
}

impl Serialize for ExportDefaultDeclarationKind<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ExportDefaultDeclarationKind::FunctionDeclaration(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::ClassDeclaration(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::BooleanLiteral(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::NullLiteral(it) => Serialize::serialize(it, serializer),
            ExportDefaultDeclarationKind::NumericLiteral(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::BigIntLiteral(it) => Serialize::serialize(it, serializer),
            ExportDefaultDeclarationKind::RegExpLiteral(it) => Serialize::serialize(it, serializer),
            ExportDefaultDeclarationKind::StringLiteral(it) => Serialize::serialize(it, serializer),
            ExportDefaultDeclarationKind::TemplateLiteral(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::Identifier(it) => Serialize::serialize(it, serializer),
            ExportDefaultDeclarationKind::MetaProperty(it) => Serialize::serialize(it, serializer),
            ExportDefaultDeclarationKind::Super(it) => Serialize::serialize(it, serializer),
            ExportDefaultDeclarationKind::ArrayExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::ArrowFunctionExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::AssignmentExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::AwaitExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::BinaryExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::CallExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::ChainExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::ClassExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::ConditionalExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::FunctionExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::ImportExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::LogicalExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::NewExpression(it) => Serialize::serialize(it, serializer),
            ExportDefaultDeclarationKind::ObjectExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::ParenthesizedExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::SequenceExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::TaggedTemplateExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::ThisExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::UnaryExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::UpdateExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::YieldExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::PrivateInExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::JSXElement(it) => Serialize::serialize(it, serializer),
            ExportDefaultDeclarationKind::JSXFragment(it) => Serialize::serialize(it, serializer),
            ExportDefaultDeclarationKind::TSAsExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::TSSatisfiesExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::TSTypeAssertion(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::TSNonNullExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::TSInstantiationExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::ComputedMemberExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::StaticMemberExpression(it) => {
                Serialize::serialize(it, serializer)
            }
            ExportDefaultDeclarationKind::PrivateFieldExpression(it) => {
                Serialize::serialize(it, serializer)
            }
        }
    }
}

impl Serialize for ModuleExportName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ModuleExportName::IdentifierName(it) => Serialize::serialize(it, serializer),
            ModuleExportName::IdentifierReference(it) => Serialize::serialize(it, serializer),
            ModuleExportName::StringLiteral(it) => Serialize::serialize(it, serializer),
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
            RegExpPattern::Raw(it) => Serialize::serialize(it, serializer),
            RegExpPattern::Invalid(it) => Serialize::serialize(it, serializer),
            RegExpPattern::Pattern(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for JSXElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXElement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

impl Serialize for JSXFragment<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXFragment")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for JSXClosingFragment {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXClosingFragment")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for JSXNamespacedName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXNamespacedName")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("namespace", &self.namespace)?;
        map.serialize_entry("property", &self.property)?;
        map.end()
    }
}

impl Serialize for JSXMemberExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXMemberExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("object", &self.object)?;
        map.serialize_entry("property", &self.property)?;
        map.end()
    }
}

impl Serialize for JSXExpressionContainer<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXExpressionContainer")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for JSXExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            JSXExpression::EmptyExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::BooleanLiteral(it) => Serialize::serialize(it, serializer),
            JSXExpression::NullLiteral(it) => Serialize::serialize(it, serializer),
            JSXExpression::NumericLiteral(it) => Serialize::serialize(it, serializer),
            JSXExpression::BigIntLiteral(it) => Serialize::serialize(it, serializer),
            JSXExpression::RegExpLiteral(it) => Serialize::serialize(it, serializer),
            JSXExpression::StringLiteral(it) => Serialize::serialize(it, serializer),
            JSXExpression::TemplateLiteral(it) => Serialize::serialize(it, serializer),
            JSXExpression::Identifier(it) => Serialize::serialize(it, serializer),
            JSXExpression::MetaProperty(it) => Serialize::serialize(it, serializer),
            JSXExpression::Super(it) => Serialize::serialize(it, serializer),
            JSXExpression::ArrayExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::ArrowFunctionExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::AssignmentExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::AwaitExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::BinaryExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::CallExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::ChainExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::ClassExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::ConditionalExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::FunctionExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::ImportExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::LogicalExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::NewExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::ObjectExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::ParenthesizedExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::SequenceExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::TaggedTemplateExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::ThisExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::UnaryExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::UpdateExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::YieldExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::PrivateInExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::JSXElement(it) => Serialize::serialize(it, serializer),
            JSXExpression::JSXFragment(it) => Serialize::serialize(it, serializer),
            JSXExpression::TSAsExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::TSSatisfiesExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::TSTypeAssertion(it) => Serialize::serialize(it, serializer),
            JSXExpression::TSNonNullExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::TSInstantiationExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::ComputedMemberExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::StaticMemberExpression(it) => Serialize::serialize(it, serializer),
            JSXExpression::PrivateFieldExpression(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for JSXEmptyExpression {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXEmptyExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for JSXAttributeItem<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            JSXAttributeItem::Attribute(it) => Serialize::serialize(it, serializer),
            JSXAttributeItem::SpreadAttribute(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for JSXAttribute<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXAttribute")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for JSXSpreadAttribute<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXSpreadAttribute")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

impl Serialize for JSXAttributeName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            JSXAttributeName::Identifier(it) => Serialize::serialize(it, serializer),
            JSXAttributeName::NamespacedName(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for JSXAttributeValue<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            JSXAttributeValue::StringLiteral(it) => Serialize::serialize(it, serializer),
            JSXAttributeValue::ExpressionContainer(it) => Serialize::serialize(it, serializer),
            JSXAttributeValue::Element(it) => Serialize::serialize(it, serializer),
            JSXAttributeValue::Fragment(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for JSXIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXIdentifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

impl Serialize for JSXChild<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            JSXChild::Text(it) => Serialize::serialize(it, serializer),
            JSXChild::Element(it) => Serialize::serialize(it, serializer),
            JSXChild::Fragment(it) => Serialize::serialize(it, serializer),
            JSXChild::ExpressionContainer(it) => Serialize::serialize(it, serializer),
            JSXChild::Spread(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for JSXSpreadChild<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXSpreadChild")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for JSXText<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXText")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for TSThisParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSThisParameter")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSEnumDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSEnumDeclaration")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("initializer", &self.initializer)?;
        map.end()
    }
}

impl Serialize for TSEnumMemberName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSEnumMemberName::Identifier(it) => Serialize::serialize(it, serializer),
            TSEnumMemberName::String(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for TSTypeAnnotation<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeAnnotation")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSLiteralType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSLiteralType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("literal", &self.literal)?;
        map.end()
    }
}

impl Serialize for TSLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSLiteral::BooleanLiteral(it) => Serialize::serialize(it, serializer),
            TSLiteral::NullLiteral(it) => Serialize::serialize(it, serializer),
            TSLiteral::NumericLiteral(it) => Serialize::serialize(it, serializer),
            TSLiteral::BigIntLiteral(it) => Serialize::serialize(it, serializer),
            TSLiteral::RegExpLiteral(it) => Serialize::serialize(it, serializer),
            TSLiteral::StringLiteral(it) => Serialize::serialize(it, serializer),
            TSLiteral::TemplateLiteral(it) => Serialize::serialize(it, serializer),
            TSLiteral::UnaryExpression(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for TSType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSType::TSAnyKeyword(it) => Serialize::serialize(it, serializer),
            TSType::TSBigIntKeyword(it) => Serialize::serialize(it, serializer),
            TSType::TSBooleanKeyword(it) => Serialize::serialize(it, serializer),
            TSType::TSIntrinsicKeyword(it) => Serialize::serialize(it, serializer),
            TSType::TSNeverKeyword(it) => Serialize::serialize(it, serializer),
            TSType::TSNullKeyword(it) => Serialize::serialize(it, serializer),
            TSType::TSNumberKeyword(it) => Serialize::serialize(it, serializer),
            TSType::TSObjectKeyword(it) => Serialize::serialize(it, serializer),
            TSType::TSStringKeyword(it) => Serialize::serialize(it, serializer),
            TSType::TSSymbolKeyword(it) => Serialize::serialize(it, serializer),
            TSType::TSUndefinedKeyword(it) => Serialize::serialize(it, serializer),
            TSType::TSUnknownKeyword(it) => Serialize::serialize(it, serializer),
            TSType::TSVoidKeyword(it) => Serialize::serialize(it, serializer),
            TSType::TSArrayType(it) => Serialize::serialize(it, serializer),
            TSType::TSConditionalType(it) => Serialize::serialize(it, serializer),
            TSType::TSConstructorType(it) => Serialize::serialize(it, serializer),
            TSType::TSFunctionType(it) => Serialize::serialize(it, serializer),
            TSType::TSImportType(it) => Serialize::serialize(it, serializer),
            TSType::TSIndexedAccessType(it) => Serialize::serialize(it, serializer),
            TSType::TSInferType(it) => Serialize::serialize(it, serializer),
            TSType::TSIntersectionType(it) => Serialize::serialize(it, serializer),
            TSType::TSLiteralType(it) => Serialize::serialize(it, serializer),
            TSType::TSMappedType(it) => Serialize::serialize(it, serializer),
            TSType::TSNamedTupleMember(it) => Serialize::serialize(it, serializer),
            TSType::TSQualifiedName(it) => Serialize::serialize(it, serializer),
            TSType::TSTemplateLiteralType(it) => Serialize::serialize(it, serializer),
            TSType::TSThisType(it) => Serialize::serialize(it, serializer),
            TSType::TSTupleType(it) => Serialize::serialize(it, serializer),
            TSType::TSTypeLiteral(it) => Serialize::serialize(it, serializer),
            TSType::TSTypeOperatorType(it) => Serialize::serialize(it, serializer),
            TSType::TSTypePredicate(it) => Serialize::serialize(it, serializer),
            TSType::TSTypeQuery(it) => Serialize::serialize(it, serializer),
            TSType::TSTypeReference(it) => Serialize::serialize(it, serializer),
            TSType::TSUnionType(it) => Serialize::serialize(it, serializer),
            TSType::TSParenthesizedType(it) => Serialize::serialize(it, serializer),
            TSType::JSDocNullableType(it) => Serialize::serialize(it, serializer),
            TSType::JSDocNonNullableType(it) => Serialize::serialize(it, serializer),
            TSType::JSDocUnknownType(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for TSConditionalType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSConditionalType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("types", &self.types)?;
        map.end()
    }
}

impl Serialize for TSIntersectionType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSIntersectionType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("types", &self.types)?;
        map.end()
    }
}

impl Serialize for TSParenthesizedType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSParenthesizedType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSTypeOperator<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeOperator")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("operator", &self.operator)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSTypeOperatorOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("elementType", &self.element_type)?;
        map.end()
    }
}

impl Serialize for TSIndexedAccessType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSIndexedAccessType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("objectType", &self.object_type)?;
        map.serialize_entry("indexType", &self.index_type)?;
        map.end()
    }
}

impl Serialize for TSTupleType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTupleType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("elementTypes", &self.element_types)?;
        map.end()
    }
}

impl Serialize for TSNamedTupleMember<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNamedTupleMember")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSRestType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSRestType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSTupleElement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSTupleElement::TSOptionalType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSRestType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSAnyKeyword(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSBigIntKeyword(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSBooleanKeyword(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSIntrinsicKeyword(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSNeverKeyword(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSNullKeyword(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSNumberKeyword(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSObjectKeyword(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSStringKeyword(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSSymbolKeyword(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSUndefinedKeyword(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSUnknownKeyword(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSVoidKeyword(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSArrayType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSConditionalType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSConstructorType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSFunctionType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSImportType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSIndexedAccessType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSInferType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSIntersectionType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSLiteralType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSMappedType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSNamedTupleMember(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSQualifiedName(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSTemplateLiteralType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSThisType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSTupleType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSTypeLiteral(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSTypeOperatorType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSTypePredicate(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSTypeQuery(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSTypeReference(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSUnionType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::TSParenthesizedType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::JSDocNullableType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::JSDocNonNullableType(it) => Serialize::serialize(it, serializer),
            TSTupleElement::JSDocUnknownType(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for TSAnyKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSAnyKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for TSStringKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSStringKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for TSBooleanKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSBooleanKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for TSNumberKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNumberKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for TSNeverKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNeverKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for TSIntrinsicKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSIntrinsicKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for TSUnknownKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSUnknownKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for TSNullKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNullKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for TSUndefinedKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSUndefinedKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for TSVoidKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSVoidKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for TSSymbolKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSSymbolKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for TSThisType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSThisType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for TSObjectKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSObjectKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for TSBigIntKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSBigIntKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

impl Serialize for TSTypeReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeReference")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeName", &self.type_name)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

impl Serialize for TSTypeName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSTypeName::IdentifierReference(it) => Serialize::serialize(it, serializer),
            TSTypeName::QualifiedName(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for TSQualifiedName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSQualifiedName")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("left", &self.left)?;
        map.serialize_entry("right", &self.right)?;
        map.end()
    }
}

impl Serialize for TSTypeParameterInstantiation<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeParameterInstantiation")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("params", &self.params)?;
        map.end()
    }
}

impl Serialize for TSTypeParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeParameter")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("params", &self.params)?;
        map.end()
    }
}

impl Serialize for TSTypeAliasDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeAliasDeclaration")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("declare", &self.declare)?;
        map.end()
    }
}

impl Serialize for TSAccessibility {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

impl Serialize for TSInterfaceDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSInterfaceDeclaration")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for TSPropertySignature<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSPropertySignature")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
            TSSignature::TSIndexSignature(it) => Serialize::serialize(it, serializer),
            TSSignature::TSPropertySignature(it) => Serialize::serialize(it, serializer),
            TSSignature::TSCallSignatureDeclaration(it) => Serialize::serialize(it, serializer),
            TSSignature::TSConstructSignatureDeclaration(it) => {
                Serialize::serialize(it, serializer)
            }
            TSSignature::TSMethodSignature(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for TSIndexSignature<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSIndexSignature")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("thisParam", &self.this_param)?;
        map.serialize_entry("params", &self.params)?;
        map.serialize_entry("returnType", &self.return_type)?;
        map.end()
    }
}

impl Serialize for TSMethodSignatureKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSInterfaceHeritage<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSInterfaceHeritage")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

impl Serialize for TSTypePredicate<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypePredicate")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("parameterName", &self.parameter_name)?;
        map.serialize_entry("asserts", &self.asserts)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSTypePredicateName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSTypePredicateName::Identifier(it) => Serialize::serialize(it, serializer),
            TSTypePredicateName::This(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for TSModuleDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSModuleDeclaration")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("body", &self.body)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("declare", &self.declare)?;
        map.end()
    }
}

impl Serialize for TSModuleDeclarationKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
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
            TSModuleDeclarationName::Identifier(it) => Serialize::serialize(it, serializer),
            TSModuleDeclarationName::StringLiteral(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for TSModuleDeclarationBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSModuleDeclarationBody::TSModuleDeclaration(it) => {
                Serialize::serialize(it, serializer)
            }
            TSModuleDeclarationBody::TSModuleBlock(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for TSTypeLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeLiteral")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("members", &self.members)?;
        map.end()
    }
}

impl Serialize for TSInferType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSInferType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeParameter", &self.type_parameter)?;
        map.end()
    }
}

impl Serialize for TSTypeQuery<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeQuery")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("exprName", &self.expr_name)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

impl Serialize for TSTypeQueryExprName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSTypeQueryExprName::TSImportType(it) => Serialize::serialize(it, serializer),
            TSTypeQueryExprName::IdentifierReference(it) => Serialize::serialize(it, serializer),
            TSTypeQueryExprName::QualifiedName(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for TSImportType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSImportType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("attributesKeyword", &self.attributes_keyword)?;
        map.serialize_entry("elements", &self.elements)?;
        map.end()
    }
}

impl Serialize for TSImportAttribute<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSImportAttribute")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for TSImportAttributeName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSImportAttributeName::Identifier(it) => Serialize::serialize(it, serializer),
            TSImportAttributeName::StringLiteral(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for TSFunctionType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSFunctionType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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
        match *self {
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("quasis", &self.quasis)?;
        map.serialize_entry("types", &self.types)?;
        map.end()
    }
}

impl Serialize for TSAsExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSAsExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSSatisfiesExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSSatisfiesExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSTypeAssertion<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeAssertion")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

impl Serialize for TSImportEqualsDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSImportEqualsDeclaration")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("moduleReference", &self.module_reference)?;
        map.serialize_entry("importKind", &self.import_kind)?;
        map.end()
    }
}

impl Serialize for TSModuleReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSModuleReference::ExternalModuleReference(it) => Serialize::serialize(it, serializer),
            TSModuleReference::IdentifierReference(it) => Serialize::serialize(it, serializer),
            TSModuleReference::QualifiedName(it) => Serialize::serialize(it, serializer),
        }
    }
}

impl Serialize for TSExternalModuleReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSExternalModuleReference")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for TSNonNullExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNonNullExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for Decorator<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Decorator")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for TSExportAssignment<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSExportAssignment")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

impl Serialize for TSNamespaceExportDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNamespaceExportDeclaration")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("id", &self.id)?;
        map.end()
    }
}

impl Serialize for TSInstantiationExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSInstantiationExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

impl Serialize for ImportOrExportKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
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
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("postfix", &self.postfix)?;
        map.end()
    }
}

impl Serialize for JSDocNonNullableType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSDocNonNullableType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("postfix", &self.postfix)?;
        map.end()
    }
}

impl Serialize for JSDocUnknownType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSDocUnknownType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}
