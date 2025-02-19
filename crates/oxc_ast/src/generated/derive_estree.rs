// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, clippy::match_same_arms)]

use serde::{__private::ser::FlatMapSerializer, ser::SerializeMap, Serialize, Serializer};

use oxc_estree::ser::{AppendTo, AppendToConcat};

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
        map.serialize_entry(
            "body",
            &AppendToConcat { array: &self.directives, after: &self.body },
        )?;
        self.source_type.serialize(FlatMapSerializer(&mut map))?;
        map.serialize_entry("hashbang", &self.hashbang)?;
        map.end()
    }
}

impl Serialize for Expression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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
            Self::ObjectProperty(it) => it.serialize(serializer),
            Self::SpreadProperty(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for PropertyKey<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for PropertyKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Init => serializer.serialize_unit_variant("PropertyKind", 0, "init"),
            Self::Get => serializer.serialize_unit_variant("PropertyKind", 1, "get"),
            Self::Set => serializer.serialize_unit_variant("PropertyKind", 2, "set"),
        }
    }
}

impl Serialize for TemplateLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TemplateLiteral")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("expressions", &self.expressions)?;
        map.serialize_entry("quasis", &self.quasis)?;
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
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("tail", &self.tail)?;
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
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
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
        map.serialize_entry("computed", &crate::serialize::True(self))?;
        map.serialize_entry("optional", &self.optional)?;
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
        map.serialize_entry("computed", &crate::serialize::False(self))?;
        map.serialize_entry("optional", &self.optional)?;
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
        map.serialize_entry("property", &self.field)?;
        map.serialize_entry("computed", &crate::serialize::False(self))?;
        map.serialize_entry("optional", &self.optional)?;
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
        map.serialize_entry("prefix", &crate::serialize::True(self))?;
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
        map.serialize_entry("type", "BinaryExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("left", &self.left)?;
        map.serialize_entry("operator", &crate::serialize::In(self))?;
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

impl Serialize for SimpleAssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl Serialize for AssignmentTargetPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::ArrayAssignmentTarget(it) => it.serialize(serializer),
            Self::ObjectAssignmentTarget(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for ArrayAssignmentTarget<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ArrayPattern")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("elements", &AppendTo { array: &self.elements, after: &self.rest })?;
        map.end()
    }
}

impl Serialize for ObjectAssignmentTarget<'_> {
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

impl Serialize for AssignmentTargetWithDefault<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AssignmentPattern")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("left", &self.binding)?;
        map.serialize_entry("right", &self.init)?;
        map.end()
    }
}

impl Serialize for AssignmentTargetProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::AssignmentTargetPropertyIdentifier(it) => it.serialize(serializer),
            Self::AssignmentTargetPropertyProperty(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for AssignmentTargetPropertyIdentifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Property")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("method", &crate::serialize::False(self))?;
        map.serialize_entry("shorthand", &crate::serialize::True(self))?;
        map.serialize_entry("computed", &crate::serialize::False(self))?;
        map.serialize_entry("key", &self.binding)?;
        map.serialize_entry("kind", &crate::serialize::Init(self))?;
        map.serialize_entry(
            "value",
            &crate::serialize::AssignmentTargetPropertyIdentifierValue(self),
        )?;
        map.end()
    }
}

impl Serialize for AssignmentTargetPropertyProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Property")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("method", &crate::serialize::False(self))?;
        map.serialize_entry("shorthand", &crate::serialize::False(self))?;
        map.serialize_entry("computed", &self.computed)?;
        map.serialize_entry("key", &self.name)?;
        map.serialize_entry("value", &self.binding)?;
        map.serialize_entry("kind", &crate::serialize::Init(self))?;
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
            Self::CallExpression(it) => it.serialize(serializer),
            Self::TSNonNullExpression(it) => it.serialize(serializer),
            Self::ComputedMemberExpression(it) => it.serialize(serializer),
            Self::StaticMemberExpression(it) => it.serialize(serializer),
            Self::PrivateFieldExpression(it) => it.serialize(serializer),
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

impl Serialize for Directive<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ExpressionStatement")?;
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

impl Serialize for VariableDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "VariableDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("declarations", &self.declarations)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("declare", &self.declare)?;
        map.end()
    }
}

impl Serialize for VariableDeclarationKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Var => serializer.serialize_unit_variant("VariableDeclarationKind", 0, "var"),
            Self::Const => serializer.serialize_unit_variant("VariableDeclarationKind", 1, "const"),
            Self::Let => serializer.serialize_unit_variant("VariableDeclarationKind", 2, "let"),
            Self::Using => serializer.serialize_unit_variant("VariableDeclarationKind", 3, "using"),
            Self::AwaitUsing => {
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
        map.serialize_entry("consequent", &self.consequent)?;
        map.serialize_entry("test", &self.test)?;
        map.end()
    }
}

impl Serialize for LabeledStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "LabeledStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("body", &self.body)?;
        map.serialize_entry("label", &self.label)?;
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
        self.pattern.kind.serialize(FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.pattern.type_annotation)?;
        map.serialize_entry("optional", &self.pattern.optional)?;
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
            Self::BindingIdentifier(it) => it.serialize(serializer),
            Self::ObjectPattern(it) => it.serialize(serializer),
            Self::ArrayPattern(it) => it.serialize(serializer),
            Self::AssignmentPattern(it) => it.serialize(serializer),
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
        map.serialize_entry("type", &self.r#type)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("expression", &crate::serialize::False(self))?;
        map.serialize_entry("generator", &self.generator)?;
        map.serialize_entry("async", &self.r#async)?;
        map.serialize_entry("params", &self.params)?;
        map.serialize_entry("body", &self.body)?;
        map.serialize_entry("declare", &self.declare)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("thisParam", &self.this_param)?;
        map.serialize_entry("returnType", &self.return_type)?;
        map.end()
    }
}

impl Serialize for FunctionType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::FunctionDeclaration => {
                serializer.serialize_unit_variant("FunctionType", 0, "FunctionDeclaration")
            }
            Self::FunctionExpression => {
                serializer.serialize_unit_variant("FunctionType", 1, "FunctionExpression")
            }
            Self::TSDeclareFunction => {
                serializer.serialize_unit_variant("FunctionType", 2, "TSDeclareFunction")
            }
            Self::TSEmptyBodyFunctionExpression => serializer.serialize_unit_variant(
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
        self.pattern.kind.serialize(FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.pattern.type_annotation)?;
        map.serialize_entry("optional", &self.pattern.optional)?;
        map.serialize_entry("decorators", &self.decorators)?;
        map.serialize_entry("accessibility", &self.accessibility)?;
        map.serialize_entry("readonly", &self.readonly)?;
        map.serialize_entry("override", &self.r#override)?;
        map.end()
    }
}

impl Serialize for FormalParameterKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::FormalParameter => {
                serializer.serialize_unit_variant("FormalParameterKind", 0, "FormalParameter")
            }
            Self::UniqueFormalParameters => serializer.serialize_unit_variant(
                "FormalParameterKind",
                1,
                "UniqueFormalParameters",
            ),
            Self::ArrowFormalParameters => {
                serializer.serialize_unit_variant("FormalParameterKind", 2, "ArrowFormalParameters")
            }
            Self::Signature => {
                serializer.serialize_unit_variant("FormalParameterKind", 3, "Signature")
            }
        }
    }
}

impl Serialize for FunctionBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BlockStatement")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry(
            "body",
            &AppendToConcat { array: &self.directives, after: &self.statements },
        )?;
        map.end()
    }
}

impl Serialize for ArrowFunctionExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ArrowFunctionExpression")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("id", &crate::serialize::Null(self))?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("generator", &crate::serialize::False(self))?;
        map.serialize_entry("async", &self.r#async)?;
        map.serialize_entry("params", &self.params)?;
        map.serialize_entry("body", &crate::serialize::ArrowFunctionExpressionBody(self))?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("returnType", &self.return_type)?;
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
        map.serialize_entry("type", &self.r#type)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("superClass", &self.super_class)?;
        map.serialize_entry("body", &self.body)?;
        map.serialize_entry("decorators", &self.decorators)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.serialize_entry("superTypeParameters", &self.super_type_parameters)?;
        map.serialize_entry("implements", &self.implements)?;
        map.serialize_entry("abstract", &self.r#abstract)?;
        map.serialize_entry("declare", &self.declare)?;
        map.end()
    }
}

impl Serialize for ClassType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::ClassDeclaration => {
                serializer.serialize_unit_variant("ClassType", 0, "ClassDeclaration")
            }
            Self::ClassExpression => {
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
            Self::StaticBlock(it) => it.serialize(serializer),
            Self::MethodDefinition(it) => it.serialize(serializer),
            Self::PropertyDefinition(it) => it.serialize(serializer),
            Self::AccessorProperty(it) => it.serialize(serializer),
            Self::TSIndexSignature(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for MethodDefinition<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", &self.r#type)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("static", &self.r#static)?;
        map.serialize_entry("computed", &self.computed)?;
        map.serialize_entry("key", &self.key)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("decorators", &self.decorators)?;
        map.serialize_entry("override", &self.r#override)?;
        map.serialize_entry("optional", &self.optional)?;
        map.serialize_entry("accessibility", &self.accessibility)?;
        map.end()
    }
}

impl Serialize for MethodDefinitionType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::MethodDefinition => {
                serializer.serialize_unit_variant("MethodDefinitionType", 0, "MethodDefinition")
            }
            Self::TSAbstractMethodDefinition => serializer.serialize_unit_variant(
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
        map.serialize_entry("type", &self.r#type)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("static", &self.r#static)?;
        map.serialize_entry("computed", &self.computed)?;
        map.serialize_entry("key", &self.key)?;
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("decorators", &self.decorators)?;
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
            Self::PropertyDefinition => {
                serializer.serialize_unit_variant("PropertyDefinitionType", 0, "PropertyDefinition")
            }
            Self::TSAbstractPropertyDefinition => serializer.serialize_unit_variant(
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
            Self::Constructor => {
                serializer.serialize_unit_variant("MethodDefinitionKind", 0, "constructor")
            }
            Self::Method => serializer.serialize_unit_variant("MethodDefinitionKind", 1, "method"),
            Self::Get => serializer.serialize_unit_variant("MethodDefinitionKind", 2, "get"),
            Self::Set => serializer.serialize_unit_variant("MethodDefinitionKind", 3, "set"),
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
            Self::ImportDeclaration(it) => it.serialize(serializer),
            Self::ExportAllDeclaration(it) => it.serialize(serializer),
            Self::ExportDefaultDeclaration(it) => it.serialize(serializer),
            Self::ExportNamedDeclaration(it) => it.serialize(serializer),
            Self::TSExportAssignment(it) => it.serialize(serializer),
            Self::TSNamespaceExportDeclaration(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for AccessorPropertyType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::AccessorProperty => {
                serializer.serialize_unit_variant("AccessorPropertyType", 0, "AccessorProperty")
            }
            Self::TSAbstractAccessorProperty => serializer.serialize_unit_variant(
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
        map.serialize_entry("type", &self.r#type)?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("key", &self.key)?;
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("computed", &self.computed)?;
        map.serialize_entry("static", &self.r#static)?;
        map.serialize_entry("decorators", &self.decorators)?;
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
        map.serialize_entry(
            "options",
            &crate::serialize::import_expression_options(&self.arguments),
        )?;
        map.end()
    }
}

impl Serialize for ImportDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ImportDeclaration")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("specifiers", &crate::serialize::OptionVecDefault(&self.specifiers))?;
        map.serialize_entry("source", &self.source)?;
        map.serialize_entry("phase", &self.phase)?;
        map.serialize_entry(
            "attributes",
            &crate::serialize::ImportExportWithClause(&self.with_clause),
        )?;
        map.serialize_entry("importKind", &self.import_kind)?;
        map.end()
    }
}

impl Serialize for ImportPhase {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Source => serializer.serialize_unit_variant("ImportPhase", 0, "source"),
            Self::Defer => serializer.serialize_unit_variant("ImportPhase", 1, "defer"),
        }
    }
}

impl Serialize for ImportDeclarationSpecifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::ImportSpecifier(it) => it.serialize(serializer),
            Self::ImportDefaultSpecifier(it) => it.serialize(serializer),
            Self::ImportNamespaceSpecifier(it) => it.serialize(serializer),
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
            Self::Identifier(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
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
        map.serialize_entry(
            "attributes",
            &crate::serialize::ImportExportWithClause(&self.with_clause),
        )?;
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
        map.serialize_entry(
            "attributes",
            &crate::serialize::ImportExportWithClause(&self.with_clause),
        )?;
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

impl Serialize for ModuleExportName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::IdentifierName(it) => it.serialize(serializer),
            Self::IdentifierReference(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for BooleanLiteral {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Literal")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("raw", &crate::serialize::BooleanLiteralRaw(self))?;
        map.end()
    }
}

impl Serialize for NullLiteral {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Literal")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("value", &crate::serialize::Null(self))?;
        map.serialize_entry("raw", &crate::serialize::NullLiteralRaw(self))?;
        map.end()
    }
}

impl Serialize for NumericLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Literal")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("raw", &self.raw)?;
        map.end()
    }
}

impl Serialize for StringLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Literal")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("raw", &self.raw)?;
        map.end()
    }
}

impl Serialize for BigIntLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Literal")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("value", &crate::serialize::BigIntLiteralValue(self))?;
        map.serialize_entry("raw", &self.raw)?;
        map.serialize_entry("bigint", &crate::serialize::BigIntLiteralBigint(self))?;
        map.end()
    }
}

impl Serialize for RegExpLiteral<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Literal")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("value", &crate::serialize::RegExpLiteralValue(self))?;
        map.serialize_entry("raw", &self.raw)?;
        map.serialize_entry("regex", &crate::serialize::RegExpLiteralRegex(self))?;
        map.end()
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
            Self::Attribute(it) => it.serialize(serializer),
            Self::SpreadAttribute(it) => it.serialize(serializer),
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
            Self::Identifier(it) => it.serialize(serializer),
            Self::NamespacedName(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for JSXAttributeValue<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::StringLiteral(it) => it.serialize(serializer),
            Self::ExpressionContainer(it) => it.serialize(serializer),
            Self::Element(it) => it.serialize(serializer),
            Self::Fragment(it) => it.serialize(serializer),
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
            Self::Text(it) => it.serialize(serializer),
            Self::Element(it) => it.serialize(serializer),
            Self::Fragment(it) => it.serialize(serializer),
            Self::ExpressionContainer(it) => it.serialize(serializer),
            Self::Spread(it) => it.serialize(serializer),
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
            Self::Identifier(it) => it.serialize(serializer),
            Self::String(it) => it.serialize(serializer),
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
            Self::BooleanLiteral(it) => it.serialize(serializer),
            Self::NumericLiteral(it) => it.serialize(serializer),
            Self::BigIntLiteral(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
            Self::TemplateLiteral(it) => it.serialize(serializer),
            Self::UnaryExpression(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for TSType<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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
            Self::Keyof => serializer.serialize_unit_variant("TSTypeOperatorOperator", 0, "keyof"),
            Self::Unique => {
                serializer.serialize_unit_variant("TSTypeOperatorOperator", 1, "unique")
            }
            Self::Readonly => {
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
            Self::IdentifierReference(it) => it.serialize(serializer),
            Self::QualifiedName(it) => it.serialize(serializer),
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
            Self::Private => serializer.serialize_unit_variant("TSAccessibility", 0, "private"),
            Self::Protected => serializer.serialize_unit_variant("TSAccessibility", 1, "protected"),
            Self::Public => serializer.serialize_unit_variant("TSAccessibility", 2, "public"),
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
            Self::TSIndexSignature(it) => it.serialize(serializer),
            Self::TSPropertySignature(it) => it.serialize(serializer),
            Self::TSCallSignatureDeclaration(it) => it.serialize(serializer),
            Self::TSConstructSignatureDeclaration(it) => it.serialize(serializer),
            Self::TSMethodSignature(it) => it.serialize(serializer),
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
            Self::Method => serializer.serialize_unit_variant("TSMethodSignatureKind", 0, "method"),
            Self::Get => serializer.serialize_unit_variant("TSMethodSignatureKind", 1, "get"),
            Self::Set => serializer.serialize_unit_variant("TSMethodSignatureKind", 2, "set"),
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
            Self::Identifier(it) => it.serialize(serializer),
            Self::This(it) => it.serialize(serializer),
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
            Self::Global => {
                serializer.serialize_unit_variant("TSModuleDeclarationKind", 0, "global")
            }
            Self::Module => {
                serializer.serialize_unit_variant("TSModuleDeclarationKind", 1, "module")
            }
            Self::Namespace => {
                serializer.serialize_unit_variant("TSModuleDeclarationKind", 2, "namespace")
            }
        }
    }
}

impl Serialize for TSModuleDeclarationName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Identifier(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for TSModuleDeclarationBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::TSModuleDeclaration(it) => it.serialize(serializer),
            Self::TSModuleBlock(it) => it.serialize(serializer),
        }
    }
}

impl Serialize for TSModuleBlock<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSModuleBlock")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry(
            "body",
            &AppendToConcat { array: &self.directives, after: &self.body },
        )?;
        map.end()
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
            Self::TSImportType(it) => it.serialize(serializer),
            Self::IdentifierReference(it) => it.serialize(serializer),
            Self::QualifiedName(it) => it.serialize(serializer),
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
            Self::Identifier(it) => it.serialize(serializer),
            Self::StringLiteral(it) => it.serialize(serializer),
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
            Self::True => {
                serializer.serialize_unit_variant("TSMappedTypeModifierOperator", 0, "true")
            }
            Self::Plus => serializer.serialize_unit_variant("TSMappedTypeModifierOperator", 1, "+"),
            Self::Minus => {
                serializer.serialize_unit_variant("TSMappedTypeModifierOperator", 2, "-")
            }
            Self::None => {
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
            Self::ExternalModuleReference(it) => it.serialize(serializer),
            Self::IdentifierReference(it) => it.serialize(serializer),
            Self::QualifiedName(it) => it.serialize(serializer),
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
            Self::Value => serializer.serialize_unit_variant("ImportOrExportKind", 0, "value"),
            Self::Type => serializer.serialize_unit_variant("ImportOrExportKind", 1, "type"),
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
