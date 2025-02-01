// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, unused_mut, clippy::match_same_arms)]

use serde::{ser::SerializeMap, Serialize, Serializer};

use crate::ast::js::*;
use crate::ast::jsx::*;
use crate::ast::literal::*;
use crate::ast::ts::*;

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
            RegExpPattern::Raw(x) => Serialize::serialize(x, serializer),
            RegExpPattern::Invalid(x) => Serialize::serialize(x, serializer),
            RegExpPattern::Pattern(x) => Serialize::serialize(x, serializer),
        }
    }
}

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
            Expression::BooleanLiteral(x) => Serialize::serialize(x, serializer),
            Expression::NullLiteral(x) => Serialize::serialize(x, serializer),
            Expression::NumericLiteral(x) => Serialize::serialize(x, serializer),
            Expression::BigIntLiteral(x) => Serialize::serialize(x, serializer),
            Expression::RegExpLiteral(x) => Serialize::serialize(x, serializer),
            Expression::StringLiteral(x) => Serialize::serialize(x, serializer),
            Expression::TemplateLiteral(x) => Serialize::serialize(x, serializer),
            Expression::Identifier(x) => Serialize::serialize(x, serializer),
            Expression::MetaProperty(x) => Serialize::serialize(x, serializer),
            Expression::Super(x) => Serialize::serialize(x, serializer),
            Expression::ArrayExpression(x) => Serialize::serialize(x, serializer),
            Expression::ArrowFunctionExpression(x) => Serialize::serialize(x, serializer),
            Expression::AssignmentExpression(x) => Serialize::serialize(x, serializer),
            Expression::AwaitExpression(x) => Serialize::serialize(x, serializer),
            Expression::BinaryExpression(x) => Serialize::serialize(x, serializer),
            Expression::CallExpression(x) => Serialize::serialize(x, serializer),
            Expression::ChainExpression(x) => Serialize::serialize(x, serializer),
            Expression::ClassExpression(x) => Serialize::serialize(x, serializer),
            Expression::ConditionalExpression(x) => Serialize::serialize(x, serializer),
            Expression::FunctionExpression(x) => Serialize::serialize(x, serializer),
            Expression::ImportExpression(x) => Serialize::serialize(x, serializer),
            Expression::LogicalExpression(x) => Serialize::serialize(x, serializer),
            Expression::NewExpression(x) => Serialize::serialize(x, serializer),
            Expression::ObjectExpression(x) => Serialize::serialize(x, serializer),
            Expression::ParenthesizedExpression(x) => Serialize::serialize(x, serializer),
            Expression::SequenceExpression(x) => Serialize::serialize(x, serializer),
            Expression::TaggedTemplateExpression(x) => Serialize::serialize(x, serializer),
            Expression::ThisExpression(x) => Serialize::serialize(x, serializer),
            Expression::UnaryExpression(x) => Serialize::serialize(x, serializer),
            Expression::UpdateExpression(x) => Serialize::serialize(x, serializer),
            Expression::YieldExpression(x) => Serialize::serialize(x, serializer),
            Expression::PrivateInExpression(x) => Serialize::serialize(x, serializer),
            Expression::JSXElement(x) => Serialize::serialize(x, serializer),
            Expression::JSXFragment(x) => Serialize::serialize(x, serializer),
            Expression::TSAsExpression(x) => Serialize::serialize(x, serializer),
            Expression::TSSatisfiesExpression(x) => Serialize::serialize(x, serializer),
            Expression::TSTypeAssertion(x) => Serialize::serialize(x, serializer),
            Expression::TSNonNullExpression(x) => Serialize::serialize(x, serializer),
            Expression::TSInstantiationExpression(x) => Serialize::serialize(x, serializer),
            Expression::ComputedMemberExpression(x) => Serialize::serialize(x, serializer),
            Expression::StaticMemberExpression(x) => Serialize::serialize(x, serializer),
            Expression::PrivateFieldExpression(x) => Serialize::serialize(x, serializer),
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
            ArrayExpressionElement::SpreadElement(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::Elision(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::BooleanLiteral(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::NullLiteral(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::NumericLiteral(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::BigIntLiteral(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::RegExpLiteral(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::StringLiteral(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::TemplateLiteral(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::Identifier(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::MetaProperty(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::Super(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::ArrayExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::ArrowFunctionExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::AssignmentExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::AwaitExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::BinaryExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::CallExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::ChainExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::ClassExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::ConditionalExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::FunctionExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::ImportExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::LogicalExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::NewExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::ObjectExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::ParenthesizedExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::SequenceExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::TaggedTemplateExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::ThisExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::UnaryExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::UpdateExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::YieldExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::PrivateInExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::JSXElement(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::JSXFragment(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::TSAsExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::TSSatisfiesExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::TSTypeAssertion(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::TSNonNullExpression(x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::TSInstantiationExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::ComputedMemberExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::StaticMemberExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::PrivateFieldExpression(x) => {
                Serialize::serialize(x, serializer)
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
            ObjectPropertyKind::ObjectProperty(x) => Serialize::serialize(x, serializer),
            ObjectPropertyKind::SpreadProperty(x) => Serialize::serialize(x, serializer),
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
            PropertyKey::StaticIdentifier(x) => Serialize::serialize(x, serializer),
            PropertyKey::PrivateIdentifier(x) => Serialize::serialize(x, serializer),
            PropertyKey::BooleanLiteral(x) => Serialize::serialize(x, serializer),
            PropertyKey::NullLiteral(x) => Serialize::serialize(x, serializer),
            PropertyKey::NumericLiteral(x) => Serialize::serialize(x, serializer),
            PropertyKey::BigIntLiteral(x) => Serialize::serialize(x, serializer),
            PropertyKey::RegExpLiteral(x) => Serialize::serialize(x, serializer),
            PropertyKey::StringLiteral(x) => Serialize::serialize(x, serializer),
            PropertyKey::TemplateLiteral(x) => Serialize::serialize(x, serializer),
            PropertyKey::Identifier(x) => Serialize::serialize(x, serializer),
            PropertyKey::MetaProperty(x) => Serialize::serialize(x, serializer),
            PropertyKey::Super(x) => Serialize::serialize(x, serializer),
            PropertyKey::ArrayExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::ArrowFunctionExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::AssignmentExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::AwaitExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::BinaryExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::CallExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::ChainExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::ClassExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::ConditionalExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::FunctionExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::ImportExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::LogicalExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::NewExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::ObjectExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::ParenthesizedExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::SequenceExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::TaggedTemplateExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::ThisExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::UnaryExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::UpdateExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::YieldExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::PrivateInExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::JSXElement(x) => Serialize::serialize(x, serializer),
            PropertyKey::JSXFragment(x) => Serialize::serialize(x, serializer),
            PropertyKey::TSAsExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::TSSatisfiesExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::TSTypeAssertion(x) => Serialize::serialize(x, serializer),
            PropertyKey::TSNonNullExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::TSInstantiationExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::ComputedMemberExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::StaticMemberExpression(x) => Serialize::serialize(x, serializer),
            PropertyKey::PrivateFieldExpression(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for PropertyKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            PropertyKind::Init => serializer.serialize_unit_variant("PropertyKind", 0u32, "init"),
            PropertyKind::Get => serializer.serialize_unit_variant("PropertyKind", 1u32, "get"),
            PropertyKind::Set => serializer.serialize_unit_variant("PropertyKind", 2u32, "set"),
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
            MemberExpression::ComputedMemberExpression(x) => Serialize::serialize(x, serializer),
            MemberExpression::StaticMemberExpression(x) => Serialize::serialize(x, serializer),
            MemberExpression::PrivateFieldExpression(x) => Serialize::serialize(x, serializer),
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
            Argument::SpreadElement(x) => Serialize::serialize(x, serializer),
            Argument::BooleanLiteral(x) => Serialize::serialize(x, serializer),
            Argument::NullLiteral(x) => Serialize::serialize(x, serializer),
            Argument::NumericLiteral(x) => Serialize::serialize(x, serializer),
            Argument::BigIntLiteral(x) => Serialize::serialize(x, serializer),
            Argument::RegExpLiteral(x) => Serialize::serialize(x, serializer),
            Argument::StringLiteral(x) => Serialize::serialize(x, serializer),
            Argument::TemplateLiteral(x) => Serialize::serialize(x, serializer),
            Argument::Identifier(x) => Serialize::serialize(x, serializer),
            Argument::MetaProperty(x) => Serialize::serialize(x, serializer),
            Argument::Super(x) => Serialize::serialize(x, serializer),
            Argument::ArrayExpression(x) => Serialize::serialize(x, serializer),
            Argument::ArrowFunctionExpression(x) => Serialize::serialize(x, serializer),
            Argument::AssignmentExpression(x) => Serialize::serialize(x, serializer),
            Argument::AwaitExpression(x) => Serialize::serialize(x, serializer),
            Argument::BinaryExpression(x) => Serialize::serialize(x, serializer),
            Argument::CallExpression(x) => Serialize::serialize(x, serializer),
            Argument::ChainExpression(x) => Serialize::serialize(x, serializer),
            Argument::ClassExpression(x) => Serialize::serialize(x, serializer),
            Argument::ConditionalExpression(x) => Serialize::serialize(x, serializer),
            Argument::FunctionExpression(x) => Serialize::serialize(x, serializer),
            Argument::ImportExpression(x) => Serialize::serialize(x, serializer),
            Argument::LogicalExpression(x) => Serialize::serialize(x, serializer),
            Argument::NewExpression(x) => Serialize::serialize(x, serializer),
            Argument::ObjectExpression(x) => Serialize::serialize(x, serializer),
            Argument::ParenthesizedExpression(x) => Serialize::serialize(x, serializer),
            Argument::SequenceExpression(x) => Serialize::serialize(x, serializer),
            Argument::TaggedTemplateExpression(x) => Serialize::serialize(x, serializer),
            Argument::ThisExpression(x) => Serialize::serialize(x, serializer),
            Argument::UnaryExpression(x) => Serialize::serialize(x, serializer),
            Argument::UpdateExpression(x) => Serialize::serialize(x, serializer),
            Argument::YieldExpression(x) => Serialize::serialize(x, serializer),
            Argument::PrivateInExpression(x) => Serialize::serialize(x, serializer),
            Argument::JSXElement(x) => Serialize::serialize(x, serializer),
            Argument::JSXFragment(x) => Serialize::serialize(x, serializer),
            Argument::TSAsExpression(x) => Serialize::serialize(x, serializer),
            Argument::TSSatisfiesExpression(x) => Serialize::serialize(x, serializer),
            Argument::TSTypeAssertion(x) => Serialize::serialize(x, serializer),
            Argument::TSNonNullExpression(x) => Serialize::serialize(x, serializer),
            Argument::TSInstantiationExpression(x) => Serialize::serialize(x, serializer),
            Argument::ComputedMemberExpression(x) => Serialize::serialize(x, serializer),
            Argument::StaticMemberExpression(x) => Serialize::serialize(x, serializer),
            Argument::PrivateFieldExpression(x) => Serialize::serialize(x, serializer),
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
            AssignmentTarget::AssignmentTargetIdentifier(x) => Serialize::serialize(x, serializer),
            AssignmentTarget::TSAsExpression(x) => Serialize::serialize(x, serializer),
            AssignmentTarget::TSSatisfiesExpression(x) => Serialize::serialize(x, serializer),
            AssignmentTarget::TSNonNullExpression(x) => Serialize::serialize(x, serializer),
            AssignmentTarget::TSTypeAssertion(x) => Serialize::serialize(x, serializer),
            AssignmentTarget::TSInstantiationExpression(x) => Serialize::serialize(x, serializer),
            AssignmentTarget::ComputedMemberExpression(x) => Serialize::serialize(x, serializer),
            AssignmentTarget::StaticMemberExpression(x) => Serialize::serialize(x, serializer),
            AssignmentTarget::PrivateFieldExpression(x) => Serialize::serialize(x, serializer),
            AssignmentTarget::ArrayAssignmentTarget(x) => Serialize::serialize(x, serializer),
            AssignmentTarget::ObjectAssignmentTarget(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for SimpleAssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(x) => {
                Serialize::serialize(x, serializer)
            }
            SimpleAssignmentTarget::TSAsExpression(x) => Serialize::serialize(x, serializer),
            SimpleAssignmentTarget::TSSatisfiesExpression(x) => Serialize::serialize(x, serializer),
            SimpleAssignmentTarget::TSNonNullExpression(x) => Serialize::serialize(x, serializer),
            SimpleAssignmentTarget::TSTypeAssertion(x) => Serialize::serialize(x, serializer),
            SimpleAssignmentTarget::TSInstantiationExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            SimpleAssignmentTarget::ComputedMemberExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            SimpleAssignmentTarget::StaticMemberExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            SimpleAssignmentTarget::PrivateFieldExpression(x) => {
                Serialize::serialize(x, serializer)
            }
        }
    }
}

impl Serialize for AssignmentTargetPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            AssignmentTargetPattern::ArrayAssignmentTarget(x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(x) => {
                Serialize::serialize(x, serializer)
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
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::TSAsExpression(x) => Serialize::serialize(x, serializer),
            AssignmentTargetMaybeDefault::TSSatisfiesExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::TSNonNullExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::TSTypeAssertion(x) => Serialize::serialize(x, serializer),
            AssignmentTargetMaybeDefault::TSInstantiationExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::ComputedMemberExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::StaticMemberExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::PrivateFieldExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::ArrayAssignmentTarget(x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(x) => {
                Serialize::serialize(x, serializer)
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
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(x) => {
                Serialize::serialize(x, serializer)
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
            ChainElement::CallExpression(x) => Serialize::serialize(x, serializer),
            ChainElement::TSNonNullExpression(x) => Serialize::serialize(x, serializer),
            ChainElement::ComputedMemberExpression(x) => Serialize::serialize(x, serializer),
            ChainElement::StaticMemberExpression(x) => Serialize::serialize(x, serializer),
            ChainElement::PrivateFieldExpression(x) => Serialize::serialize(x, serializer),
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
            Statement::BlockStatement(x) => Serialize::serialize(x, serializer),
            Statement::BreakStatement(x) => Serialize::serialize(x, serializer),
            Statement::ContinueStatement(x) => Serialize::serialize(x, serializer),
            Statement::DebuggerStatement(x) => Serialize::serialize(x, serializer),
            Statement::DoWhileStatement(x) => Serialize::serialize(x, serializer),
            Statement::EmptyStatement(x) => Serialize::serialize(x, serializer),
            Statement::ExpressionStatement(x) => Serialize::serialize(x, serializer),
            Statement::ForInStatement(x) => Serialize::serialize(x, serializer),
            Statement::ForOfStatement(x) => Serialize::serialize(x, serializer),
            Statement::ForStatement(x) => Serialize::serialize(x, serializer),
            Statement::IfStatement(x) => Serialize::serialize(x, serializer),
            Statement::LabeledStatement(x) => Serialize::serialize(x, serializer),
            Statement::ReturnStatement(x) => Serialize::serialize(x, serializer),
            Statement::SwitchStatement(x) => Serialize::serialize(x, serializer),
            Statement::ThrowStatement(x) => Serialize::serialize(x, serializer),
            Statement::TryStatement(x) => Serialize::serialize(x, serializer),
            Statement::WhileStatement(x) => Serialize::serialize(x, serializer),
            Statement::WithStatement(x) => Serialize::serialize(x, serializer),
            Statement::VariableDeclaration(x) => Serialize::serialize(x, serializer),
            Statement::FunctionDeclaration(x) => Serialize::serialize(x, serializer),
            Statement::ClassDeclaration(x) => Serialize::serialize(x, serializer),
            Statement::TSTypeAliasDeclaration(x) => Serialize::serialize(x, serializer),
            Statement::TSInterfaceDeclaration(x) => Serialize::serialize(x, serializer),
            Statement::TSEnumDeclaration(x) => Serialize::serialize(x, serializer),
            Statement::TSModuleDeclaration(x) => Serialize::serialize(x, serializer),
            Statement::TSImportEqualsDeclaration(x) => Serialize::serialize(x, serializer),
            Statement::ImportDeclaration(x) => Serialize::serialize(x, serializer),
            Statement::ExportAllDeclaration(x) => Serialize::serialize(x, serializer),
            Statement::ExportDefaultDeclaration(x) => Serialize::serialize(x, serializer),
            Statement::ExportNamedDeclaration(x) => Serialize::serialize(x, serializer),
            Statement::TSExportAssignment(x) => Serialize::serialize(x, serializer),
            Statement::TSNamespaceExportDeclaration(x) => Serialize::serialize(x, serializer),
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
            Declaration::VariableDeclaration(x) => Serialize::serialize(x, serializer),
            Declaration::FunctionDeclaration(x) => Serialize::serialize(x, serializer),
            Declaration::ClassDeclaration(x) => Serialize::serialize(x, serializer),
            Declaration::TSTypeAliasDeclaration(x) => Serialize::serialize(x, serializer),
            Declaration::TSInterfaceDeclaration(x) => Serialize::serialize(x, serializer),
            Declaration::TSEnumDeclaration(x) => Serialize::serialize(x, serializer),
            Declaration::TSModuleDeclaration(x) => Serialize::serialize(x, serializer),
            Declaration::TSImportEqualsDeclaration(x) => Serialize::serialize(x, serializer),
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
                serializer.serialize_unit_variant("VariableDeclarationKind", 0u32, "var")
            }
            VariableDeclarationKind::Const => {
                serializer.serialize_unit_variant("VariableDeclarationKind", 1u32, "const")
            }
            VariableDeclarationKind::Let => {
                serializer.serialize_unit_variant("VariableDeclarationKind", 2u32, "let")
            }
            VariableDeclarationKind::Using => {
                serializer.serialize_unit_variant("VariableDeclarationKind", 3u32, "using")
            }
            VariableDeclarationKind::AwaitUsing => {
                serializer.serialize_unit_variant("VariableDeclarationKind", 4u32, "await using")
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
            ForStatementInit::VariableDeclaration(x) => Serialize::serialize(x, serializer),
            ForStatementInit::BooleanLiteral(x) => Serialize::serialize(x, serializer),
            ForStatementInit::NullLiteral(x) => Serialize::serialize(x, serializer),
            ForStatementInit::NumericLiteral(x) => Serialize::serialize(x, serializer),
            ForStatementInit::BigIntLiteral(x) => Serialize::serialize(x, serializer),
            ForStatementInit::RegExpLiteral(x) => Serialize::serialize(x, serializer),
            ForStatementInit::StringLiteral(x) => Serialize::serialize(x, serializer),
            ForStatementInit::TemplateLiteral(x) => Serialize::serialize(x, serializer),
            ForStatementInit::Identifier(x) => Serialize::serialize(x, serializer),
            ForStatementInit::MetaProperty(x) => Serialize::serialize(x, serializer),
            ForStatementInit::Super(x) => Serialize::serialize(x, serializer),
            ForStatementInit::ArrayExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::ArrowFunctionExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::AssignmentExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::AwaitExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::BinaryExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::CallExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::ChainExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::ClassExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::ConditionalExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::FunctionExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::ImportExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::LogicalExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::NewExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::ObjectExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::ParenthesizedExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::SequenceExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::TaggedTemplateExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::ThisExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::UnaryExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::UpdateExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::YieldExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::PrivateInExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::JSXElement(x) => Serialize::serialize(x, serializer),
            ForStatementInit::JSXFragment(x) => Serialize::serialize(x, serializer),
            ForStatementInit::TSAsExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::TSSatisfiesExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::TSTypeAssertion(x) => Serialize::serialize(x, serializer),
            ForStatementInit::TSNonNullExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::TSInstantiationExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::ComputedMemberExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::StaticMemberExpression(x) => Serialize::serialize(x, serializer),
            ForStatementInit::PrivateFieldExpression(x) => Serialize::serialize(x, serializer),
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
            ForStatementLeft::VariableDeclaration(x) => Serialize::serialize(x, serializer),
            ForStatementLeft::AssignmentTargetIdentifier(x) => Serialize::serialize(x, serializer),
            ForStatementLeft::TSAsExpression(x) => Serialize::serialize(x, serializer),
            ForStatementLeft::TSSatisfiesExpression(x) => Serialize::serialize(x, serializer),
            ForStatementLeft::TSNonNullExpression(x) => Serialize::serialize(x, serializer),
            ForStatementLeft::TSTypeAssertion(x) => Serialize::serialize(x, serializer),
            ForStatementLeft::TSInstantiationExpression(x) => Serialize::serialize(x, serializer),
            ForStatementLeft::ComputedMemberExpression(x) => Serialize::serialize(x, serializer),
            ForStatementLeft::StaticMemberExpression(x) => Serialize::serialize(x, serializer),
            ForStatementLeft::PrivateFieldExpression(x) => Serialize::serialize(x, serializer),
            ForStatementLeft::ArrayAssignmentTarget(x) => Serialize::serialize(x, serializer),
            ForStatementLeft::ObjectAssignmentTarget(x) => Serialize::serialize(x, serializer),
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
            BindingPatternKind::BindingIdentifier(x) => Serialize::serialize(x, serializer),
            BindingPatternKind::ObjectPattern(x) => Serialize::serialize(x, serializer),
            BindingPatternKind::ArrayPattern(x) => Serialize::serialize(x, serializer),
            BindingPatternKind::AssignmentPattern(x) => Serialize::serialize(x, serializer),
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
                serializer.serialize_unit_variant("FunctionType", 0u32, "FunctionDeclaration")
            }
            FunctionType::FunctionExpression => {
                serializer.serialize_unit_variant("FunctionType", 1u32, "FunctionExpression")
            }
            FunctionType::TSDeclareFunction => {
                serializer.serialize_unit_variant("FunctionType", 2u32, "TSDeclareFunction")
            }
            FunctionType::TSEmptyBodyFunctionExpression => serializer.serialize_unit_variant(
                "FunctionType",
                3u32,
                "TSEmptyBodyFunctionExpression",
            ),
        }
    }
}

impl Serialize for FormalParameter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "FormalParameter")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("decorators", &self.decorators)?;
        map.serialize_entry("pattern", &self.pattern)?;
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
                serializer.serialize_unit_variant("FormalParameterKind", 0u32, "FormalParameter")
            }
            FormalParameterKind::UniqueFormalParameters => serializer.serialize_unit_variant(
                "FormalParameterKind",
                1u32,
                "UniqueFormalParameters",
            ),
            FormalParameterKind::ArrowFormalParameters => serializer.serialize_unit_variant(
                "FormalParameterKind",
                2u32,
                "ArrowFormalParameters",
            ),
            FormalParameterKind::Signature => {
                serializer.serialize_unit_variant("FormalParameterKind", 3u32, "Signature")
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
                serializer.serialize_unit_variant("ClassType", 0u32, "ClassDeclaration")
            }
            ClassType::ClassExpression => {
                serializer.serialize_unit_variant("ClassType", 1u32, "ClassExpression")
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
            ClassElement::StaticBlock(x) => Serialize::serialize(x, serializer),
            ClassElement::MethodDefinition(x) => Serialize::serialize(x, serializer),
            ClassElement::PropertyDefinition(x) => Serialize::serialize(x, serializer),
            ClassElement::AccessorProperty(x) => Serialize::serialize(x, serializer),
            ClassElement::TSIndexSignature(x) => Serialize::serialize(x, serializer),
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
                serializer.serialize_unit_variant("MethodDefinitionType", 0u32, "MethodDefinition")
            }
            MethodDefinitionType::TSAbstractMethodDefinition => serializer.serialize_unit_variant(
                "MethodDefinitionType",
                1u32,
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
            PropertyDefinitionType::PropertyDefinition => serializer.serialize_unit_variant(
                "PropertyDefinitionType",
                0u32,
                "PropertyDefinition",
            ),
            PropertyDefinitionType::TSAbstractPropertyDefinition => serializer
                .serialize_unit_variant(
                    "PropertyDefinitionType",
                    1u32,
                    "TSAbstractPropertyDefinition",
                ),
        }
    }
}

impl Serialize for MethodDefinitionKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            MethodDefinitionKind::Constructor => {
                serializer.serialize_unit_variant("MethodDefinitionKind", 0u32, "constructor")
            }
            MethodDefinitionKind::Method => {
                serializer.serialize_unit_variant("MethodDefinitionKind", 1u32, "method")
            }
            MethodDefinitionKind::Get => {
                serializer.serialize_unit_variant("MethodDefinitionKind", 2u32, "get")
            }
            MethodDefinitionKind::Set => {
                serializer.serialize_unit_variant("MethodDefinitionKind", 3u32, "set")
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
            ModuleDeclaration::ImportDeclaration(x) => Serialize::serialize(x, serializer),
            ModuleDeclaration::ExportAllDeclaration(x) => Serialize::serialize(x, serializer),
            ModuleDeclaration::ExportDefaultDeclaration(x) => Serialize::serialize(x, serializer),
            ModuleDeclaration::ExportNamedDeclaration(x) => Serialize::serialize(x, serializer),
            ModuleDeclaration::TSExportAssignment(x) => Serialize::serialize(x, serializer),
            ModuleDeclaration::TSNamespaceExportDeclaration(x) => {
                Serialize::serialize(x, serializer)
            }
        }
    }
}

impl Serialize for AccessorPropertyType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            AccessorPropertyType::AccessorProperty => {
                serializer.serialize_unit_variant("AccessorPropertyType", 0u32, "AccessorProperty")
            }
            AccessorPropertyType::TSAbstractAccessorProperty => serializer.serialize_unit_variant(
                "AccessorPropertyType",
                1u32,
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
        map.serialize_entry("specifiers", &crate::serialize::OptionVecDefault(&self.specifiers))?;
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
            ImportPhase::Source => serializer.serialize_unit_variant("ImportPhase", 0u32, "source"),
            ImportPhase::Defer => serializer.serialize_unit_variant("ImportPhase", 1u32, "defer"),
        }
    }
}

impl Serialize for ImportDeclarationSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ImportDeclarationSpecifier::ImportSpecifier(x) => Serialize::serialize(x, serializer),
            ImportDeclarationSpecifier::ImportDefaultSpecifier(x) => {
                Serialize::serialize(x, serializer)
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(x) => {
                Serialize::serialize(x, serializer)
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
            ImportAttributeKey::Identifier(x) => Serialize::serialize(x, serializer),
            ImportAttributeKey::StringLiteral(x) => Serialize::serialize(x, serializer),
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
            ExportDefaultDeclarationKind::FunctionDeclaration(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::ClassDeclaration(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::BooleanLiteral(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::NullLiteral(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::NumericLiteral(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::BigIntLiteral(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::RegExpLiteral(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::StringLiteral(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::TemplateLiteral(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::Identifier(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::MetaProperty(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::Super(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::ArrayExpression(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::ArrowFunctionExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::AssignmentExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::AwaitExpression(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::BinaryExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::CallExpression(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::ChainExpression(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::ClassExpression(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::ConditionalExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::FunctionExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::ImportExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::LogicalExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::NewExpression(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::ObjectExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::ParenthesizedExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::SequenceExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::TaggedTemplateExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::ThisExpression(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::UnaryExpression(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::UpdateExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::YieldExpression(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::PrivateInExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::JSXElement(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::JSXFragment(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::TSAsExpression(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::TSSatisfiesExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::TSTypeAssertion(x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::TSNonNullExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::TSInstantiationExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::ComputedMemberExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::StaticMemberExpression(x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::PrivateFieldExpression(x) => {
                Serialize::serialize(x, serializer)
            }
        }
    }
}

impl Serialize for ModuleExportName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ModuleExportName::IdentifierName(x) => Serialize::serialize(x, serializer),
            ModuleExportName::IdentifierReference(x) => Serialize::serialize(x, serializer),
            ModuleExportName::StringLiteral(x) => Serialize::serialize(x, serializer),
        }
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
            TSEnumMemberName::Identifier(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::String(x) => Serialize::serialize(x, serializer),
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
            TSLiteral::BooleanLiteral(x) => Serialize::serialize(x, serializer),
            TSLiteral::NullLiteral(x) => Serialize::serialize(x, serializer),
            TSLiteral::NumericLiteral(x) => Serialize::serialize(x, serializer),
            TSLiteral::BigIntLiteral(x) => Serialize::serialize(x, serializer),
            TSLiteral::RegExpLiteral(x) => Serialize::serialize(x, serializer),
            TSLiteral::StringLiteral(x) => Serialize::serialize(x, serializer),
            TSLiteral::TemplateLiteral(x) => Serialize::serialize(x, serializer),
            TSLiteral::UnaryExpression(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for TSType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSType::TSAnyKeyword(x) => Serialize::serialize(x, serializer),
            TSType::TSBigIntKeyword(x) => Serialize::serialize(x, serializer),
            TSType::TSBooleanKeyword(x) => Serialize::serialize(x, serializer),
            TSType::TSIntrinsicKeyword(x) => Serialize::serialize(x, serializer),
            TSType::TSNeverKeyword(x) => Serialize::serialize(x, serializer),
            TSType::TSNullKeyword(x) => Serialize::serialize(x, serializer),
            TSType::TSNumberKeyword(x) => Serialize::serialize(x, serializer),
            TSType::TSObjectKeyword(x) => Serialize::serialize(x, serializer),
            TSType::TSStringKeyword(x) => Serialize::serialize(x, serializer),
            TSType::TSSymbolKeyword(x) => Serialize::serialize(x, serializer),
            TSType::TSUndefinedKeyword(x) => Serialize::serialize(x, serializer),
            TSType::TSUnknownKeyword(x) => Serialize::serialize(x, serializer),
            TSType::TSVoidKeyword(x) => Serialize::serialize(x, serializer),
            TSType::TSArrayType(x) => Serialize::serialize(x, serializer),
            TSType::TSConditionalType(x) => Serialize::serialize(x, serializer),
            TSType::TSConstructorType(x) => Serialize::serialize(x, serializer),
            TSType::TSFunctionType(x) => Serialize::serialize(x, serializer),
            TSType::TSImportType(x) => Serialize::serialize(x, serializer),
            TSType::TSIndexedAccessType(x) => Serialize::serialize(x, serializer),
            TSType::TSInferType(x) => Serialize::serialize(x, serializer),
            TSType::TSIntersectionType(x) => Serialize::serialize(x, serializer),
            TSType::TSLiteralType(x) => Serialize::serialize(x, serializer),
            TSType::TSMappedType(x) => Serialize::serialize(x, serializer),
            TSType::TSNamedTupleMember(x) => Serialize::serialize(x, serializer),
            TSType::TSQualifiedName(x) => Serialize::serialize(x, serializer),
            TSType::TSTemplateLiteralType(x) => Serialize::serialize(x, serializer),
            TSType::TSThisType(x) => Serialize::serialize(x, serializer),
            TSType::TSTupleType(x) => Serialize::serialize(x, serializer),
            TSType::TSTypeLiteral(x) => Serialize::serialize(x, serializer),
            TSType::TSTypeOperatorType(x) => Serialize::serialize(x, serializer),
            TSType::TSTypePredicate(x) => Serialize::serialize(x, serializer),
            TSType::TSTypeQuery(x) => Serialize::serialize(x, serializer),
            TSType::TSTypeReference(x) => Serialize::serialize(x, serializer),
            TSType::TSUnionType(x) => Serialize::serialize(x, serializer),
            TSType::TSParenthesizedType(x) => Serialize::serialize(x, serializer),
            TSType::JSDocNullableType(x) => Serialize::serialize(x, serializer),
            TSType::JSDocNonNullableType(x) => Serialize::serialize(x, serializer),
            TSType::JSDocUnknownType(x) => Serialize::serialize(x, serializer),
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
                serializer.serialize_unit_variant("TSTypeOperatorOperator", 0u32, "keyof")
            }
            TSTypeOperatorOperator::Unique => {
                serializer.serialize_unit_variant("TSTypeOperatorOperator", 1u32, "unique")
            }
            TSTypeOperatorOperator::Readonly => {
                serializer.serialize_unit_variant("TSTypeOperatorOperator", 2u32, "readonly")
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
            TSTupleElement::TSOptionalType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSRestType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSAnyKeyword(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSBigIntKeyword(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSBooleanKeyword(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSIntrinsicKeyword(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSNeverKeyword(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSNullKeyword(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSNumberKeyword(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSObjectKeyword(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSStringKeyword(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSSymbolKeyword(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSUndefinedKeyword(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSUnknownKeyword(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSVoidKeyword(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSArrayType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSConditionalType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSConstructorType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSFunctionType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSImportType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSIndexedAccessType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSInferType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSIntersectionType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSLiteralType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSMappedType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSNamedTupleMember(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSQualifiedName(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSTemplateLiteralType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSThisType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSTupleType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSTypeLiteral(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSTypeOperatorType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSTypePredicate(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSTypeQuery(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSTypeReference(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSUnionType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSParenthesizedType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::JSDocNullableType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::JSDocNonNullableType(x) => Serialize::serialize(x, serializer),
            TSTupleElement::JSDocUnknownType(x) => Serialize::serialize(x, serializer),
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
            TSTypeName::IdentifierReference(x) => Serialize::serialize(x, serializer),
            TSTypeName::QualifiedName(x) => Serialize::serialize(x, serializer),
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
                serializer.serialize_unit_variant("TSAccessibility", 0u32, "private")
            }
            TSAccessibility::Protected => {
                serializer.serialize_unit_variant("TSAccessibility", 1u32, "protected")
            }
            TSAccessibility::Public => {
                serializer.serialize_unit_variant("TSAccessibility", 2u32, "public")
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
            TSSignature::TSIndexSignature(x) => Serialize::serialize(x, serializer),
            TSSignature::TSPropertySignature(x) => Serialize::serialize(x, serializer),
            TSSignature::TSCallSignatureDeclaration(x) => Serialize::serialize(x, serializer),
            TSSignature::TSConstructSignatureDeclaration(x) => Serialize::serialize(x, serializer),
            TSSignature::TSMethodSignature(x) => Serialize::serialize(x, serializer),
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
                serializer.serialize_unit_variant("TSMethodSignatureKind", 0u32, "method")
            }
            TSMethodSignatureKind::Get => {
                serializer.serialize_unit_variant("TSMethodSignatureKind", 1u32, "get")
            }
            TSMethodSignatureKind::Set => {
                serializer.serialize_unit_variant("TSMethodSignatureKind", 2u32, "set")
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
            TSTypePredicateName::Identifier(x) => Serialize::serialize(x, serializer),
            TSTypePredicateName::This(x) => Serialize::serialize(x, serializer),
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
                serializer.serialize_unit_variant("TSModuleDeclarationKind", 0u32, "global")
            }
            TSModuleDeclarationKind::Module => {
                serializer.serialize_unit_variant("TSModuleDeclarationKind", 1u32, "module")
            }
            TSModuleDeclarationKind::Namespace => {
                serializer.serialize_unit_variant("TSModuleDeclarationKind", 2u32, "namespace")
            }
        }
    }
}

impl Serialize for TSModuleDeclarationName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSModuleDeclarationName::Identifier(x) => Serialize::serialize(x, serializer),
            TSModuleDeclarationName::StringLiteral(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for TSModuleDeclarationBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSModuleDeclarationBody::TSModuleDeclaration(x) => Serialize::serialize(x, serializer),
            TSModuleDeclarationBody::TSModuleBlock(x) => Serialize::serialize(x, serializer),
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
            TSTypeQueryExprName::TSImportType(x) => Serialize::serialize(x, serializer),
            TSTypeQueryExprName::IdentifierReference(x) => Serialize::serialize(x, serializer),
            TSTypeQueryExprName::QualifiedName(x) => Serialize::serialize(x, serializer),
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
            TSImportAttributeName::Identifier(x) => Serialize::serialize(x, serializer),
            TSImportAttributeName::StringLiteral(x) => Serialize::serialize(x, serializer),
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
                serializer.serialize_unit_variant("TSMappedTypeModifierOperator", 0u32, "true")
            }
            TSMappedTypeModifierOperator::Plus => {
                serializer.serialize_unit_variant("TSMappedTypeModifierOperator", 1u32, "+")
            }
            TSMappedTypeModifierOperator::Minus => {
                serializer.serialize_unit_variant("TSMappedTypeModifierOperator", 2u32, "-")
            }
            TSMappedTypeModifierOperator::None => {
                serializer.serialize_unit_variant("TSMappedTypeModifierOperator", 3u32, "none")
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
            TSModuleReference::ExternalModuleReference(x) => Serialize::serialize(x, serializer),
            TSModuleReference::IdentifierReference(x) => Serialize::serialize(x, serializer),
            TSModuleReference::QualifiedName(x) => Serialize::serialize(x, serializer),
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
                serializer.serialize_unit_variant("ImportOrExportKind", 0u32, "value")
            }
            ImportOrExportKind::Type => {
                serializer.serialize_unit_variant("ImportOrExportKind", 1u32, "type")
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
            JSXExpression::EmptyExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::BooleanLiteral(x) => Serialize::serialize(x, serializer),
            JSXExpression::NullLiteral(x) => Serialize::serialize(x, serializer),
            JSXExpression::NumericLiteral(x) => Serialize::serialize(x, serializer),
            JSXExpression::BigIntLiteral(x) => Serialize::serialize(x, serializer),
            JSXExpression::RegExpLiteral(x) => Serialize::serialize(x, serializer),
            JSXExpression::StringLiteral(x) => Serialize::serialize(x, serializer),
            JSXExpression::TemplateLiteral(x) => Serialize::serialize(x, serializer),
            JSXExpression::Identifier(x) => Serialize::serialize(x, serializer),
            JSXExpression::MetaProperty(x) => Serialize::serialize(x, serializer),
            JSXExpression::Super(x) => Serialize::serialize(x, serializer),
            JSXExpression::ArrayExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::ArrowFunctionExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::AssignmentExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::AwaitExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::BinaryExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::CallExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::ChainExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::ClassExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::ConditionalExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::FunctionExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::ImportExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::LogicalExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::NewExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::ObjectExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::ParenthesizedExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::SequenceExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::TaggedTemplateExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::ThisExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::UnaryExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::UpdateExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::YieldExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::PrivateInExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::JSXElement(x) => Serialize::serialize(x, serializer),
            JSXExpression::JSXFragment(x) => Serialize::serialize(x, serializer),
            JSXExpression::TSAsExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::TSSatisfiesExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::TSTypeAssertion(x) => Serialize::serialize(x, serializer),
            JSXExpression::TSNonNullExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::TSInstantiationExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::ComputedMemberExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::StaticMemberExpression(x) => Serialize::serialize(x, serializer),
            JSXExpression::PrivateFieldExpression(x) => Serialize::serialize(x, serializer),
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
            JSXAttributeItem::Attribute(x) => Serialize::serialize(x, serializer),
            JSXAttributeItem::SpreadAttribute(x) => Serialize::serialize(x, serializer),
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
            JSXAttributeName::Identifier(x) => Serialize::serialize(x, serializer),
            JSXAttributeName::NamespacedName(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for JSXAttributeValue<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            JSXAttributeValue::StringLiteral(x) => Serialize::serialize(x, serializer),
            JSXAttributeValue::ExpressionContainer(x) => Serialize::serialize(x, serializer),
            JSXAttributeValue::Element(x) => Serialize::serialize(x, serializer),
            JSXAttributeValue::Fragment(x) => Serialize::serialize(x, serializer),
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
            JSXChild::Text(x) => Serialize::serialize(x, serializer),
            JSXChild::Element(x) => Serialize::serialize(x, serializer),
            JSXChild::Fragment(x) => Serialize::serialize(x, serializer),
            JSXChild::ExpressionContainer(x) => Serialize::serialize(x, serializer),
            JSXChild::Spread(x) => Serialize::serialize(x, serializer),
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
