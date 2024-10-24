// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#![allow(unused_imports, unused_mut, clippy::match_same_arms)]

use serde::{ser::SerializeMap, Serialize, Serializer};

#[allow(clippy::wildcard_imports)]
use crate::ast::js::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::jsx::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::literal::*;

#[allow(clippy::wildcard_imports)]
use crate::ast::ts::*;

impl Serialize for BooleanLiteral {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BooleanLiteral")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type BooleanLiteral = ({\n\ttype: 'BooleanLiteral';\n\tvalue: boolean;\n}) & Span;";

impl Serialize for NullLiteral {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "NullLiteral")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type NullLiteral = ({\n\ttype: 'NullLiteral';\n}) & Span;";

impl<'a> Serialize for NumericLiteral<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "NumericLiteral")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("raw", &self.raw)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type NumericLiteral = ({\n\ttype: 'NumericLiteral';\n\tvalue: number;\n\traw: string;\n}) & Span;";

impl<'a> Serialize for BigIntLiteral<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BigIntLiteral")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("raw", &self.raw)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type BigIntLiteral = ({\n\ttype: 'BigIntLiteral';\n\traw: string;\n}) & Span;";

impl<'a> Serialize for RegExpLiteral<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "RegExpLiteral")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("regex", &self.regex)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type RegExpLiteral = ({\n\ttype: 'RegExpLiteral';\n\tvalue: EmptyObject;\n\tregex: RegExp;\n}) & Span;";

impl<'a> Serialize for RegExp<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("pattern", &self.pattern)?;
        map.serialize_entry("flags", &self.flags)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type RegExp = ({\n\tpattern: RegExpPattern;\n\tflags: RegExpFlags;\n});";

impl<'a> Serialize for RegExpPattern<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            RegExpPattern::Raw(x) => Serialize::serialize(x, serializer),
            RegExpPattern::Invalid(x) => Serialize::serialize(x, serializer),
            RegExpPattern::Pattern(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type RegExpPattern = string | string | Pattern;";

impl Serialize for EmptyObject {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type EmptyObject = ({\n});";

impl<'a> Serialize for StringLiteral<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "StringLiteral")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type StringLiteral = ({\n\ttype: 'StringLiteral';\n\tvalue: string;\n}) & Span;";

impl<'a> Serialize for Program<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Program = ({\n\ttype: 'Program';\n\tsourceType: SourceType;\n\thashbang: (Hashbang) | null;\n\tdirectives: Array<Directive>;\n\tbody: Array<Statement>;\n}) & Span;";

impl<'a> Serialize for Expression<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Expression = BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for IdentifierName<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Identifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type IdentifierName = ({\n\ttype: 'Identifier';\n\tname: string;\n}) & Span;";

impl<'a> Serialize for IdentifierReference<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Identifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type IdentifierReference = ({\n\ttype: 'Identifier';\n\tname: string;\n}) & Span;";

impl<'a> Serialize for BindingIdentifier<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Identifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type BindingIdentifier = ({\n\ttype: 'Identifier';\n\tname: string;\n}) & Span;";

impl<'a> Serialize for LabelIdentifier<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Identifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type LabelIdentifier = ({\n\ttype: 'Identifier';\n\tname: string;\n}) & Span;";

impl Serialize for ThisExpression {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ThisExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type ThisExpression = ({\n\ttype: 'ThisExpression';\n}) & Span;";

impl<'a> Serialize for ArrayExpression<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ArrayExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("elements", &self.elements)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ArrayExpression = ({\n\ttype: 'ArrayExpression';\n\telements: Array<SpreadElement | Expression | null>;\n}) & Span;";

impl<'a> Serialize for ArrayExpressionElement<'a> {
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

impl<'a> Serialize for ObjectExpression<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ObjectExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("properties", &self.properties)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ObjectExpression = ({\n\ttype: 'ObjectExpression';\n\tproperties: Array<ObjectPropertyKind>;\n}) & Span;";

impl<'a> Serialize for ObjectPropertyKind<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ObjectPropertyKind::ObjectProperty(x) => Serialize::serialize(x, serializer),
            ObjectPropertyKind::SpreadProperty(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type ObjectPropertyKind = ObjectProperty | SpreadElement;";

impl<'a> Serialize for ObjectProperty<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ObjectProperty")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("key", &self.key)?;
        map.serialize_entry("value", &self.value)?;
        map.serialize_entry("init", &self.init)?;
        map.serialize_entry("method", &self.method)?;
        map.serialize_entry("shorthand", &self.shorthand)?;
        map.serialize_entry("computed", &self.computed)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ObjectProperty = ({\n\ttype: 'ObjectProperty';\n\tkind: PropertyKind;\n\tkey: PropertyKey;\n\tvalue: Expression;\n\tinit: (Expression) | null;\n\tmethod: boolean;\n\tshorthand: boolean;\n\tcomputed: boolean;\n}) & Span;";

impl<'a> Serialize for PropertyKey<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type PropertyKey = IdentifierName | PrivateIdentifier | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl Serialize for PropertyKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            PropertyKind::Init => serializer.serialize_unit_variant("PropertyKind", 0u32, "init"),
            PropertyKind::Get => serializer.serialize_unit_variant("PropertyKind", 1u32, "get"),
            PropertyKind::Set => serializer.serialize_unit_variant("PropertyKind", 2u32, "set"),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type PropertyKind = 'init' | 'get' | 'set';";

impl<'a> Serialize for TemplateLiteral<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TemplateLiteral")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("quasis", &self.quasis)?;
        map.serialize_entry("expressions", &self.expressions)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TemplateLiteral = ({\n\ttype: 'TemplateLiteral';\n\tquasis: Array<TemplateElement>;\n\texpressions: Array<Expression>;\n}) & Span;";

impl<'a> Serialize for TaggedTemplateExpression<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TaggedTemplateExpression = ({\n\ttype: 'TaggedTemplateExpression';\n\ttag: Expression;\n\tquasi: TemplateLiteral;\n\ttypeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

impl<'a> Serialize for TemplateElement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TemplateElement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("tail", &self.tail)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TemplateElement = ({\n\ttype: 'TemplateElement';\n\ttail: boolean;\n\tvalue: TemplateElementValue;\n}) & Span;";

impl<'a> Serialize for TemplateElementValue<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("raw", &self.raw)?;
        map.serialize_entry("cooked", &self.cooked)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TemplateElementValue = ({\n\traw: string;\n\tcooked: (string) | null;\n});";

impl<'a> Serialize for MemberExpression<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            MemberExpression::ComputedMemberExpression(x) => Serialize::serialize(x, serializer),
            MemberExpression::StaticMemberExpression(x) => Serialize::serialize(x, serializer),
            MemberExpression::PrivateFieldExpression(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type MemberExpression = ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for ComputedMemberExpression<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ComputedMemberExpression = ({\n\ttype: 'ComputedMemberExpression';\n\tobject: Expression;\n\texpression: Expression;\n\toptional: boolean;\n}) & Span;";

impl<'a> Serialize for StaticMemberExpression<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type StaticMemberExpression = ({\n\ttype: 'StaticMemberExpression';\n\tobject: Expression;\n\tproperty: IdentifierName;\n\toptional: boolean;\n}) & Span;";

impl<'a> Serialize for PrivateFieldExpression<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type PrivateFieldExpression = ({\n\ttype: 'PrivateFieldExpression';\n\tobject: Expression;\n\tfield: PrivateIdentifier;\n\toptional: boolean;\n}) & Span;";

impl<'a> Serialize for CallExpression<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type CallExpression = ({\n\ttype: 'CallExpression';\n\tcallee: Expression;\n\ttypeParameters: (TSTypeParameterInstantiation) | null;\n\targuments: Array<Argument>;\n\toptional: boolean;\n}) & Span;";

impl<'a> Serialize for NewExpression<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type NewExpression = ({\n\ttype: 'NewExpression';\n\tcallee: Expression;\n\targuments: Array<Argument>;\n\ttypeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

impl<'a> Serialize for MetaProperty<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "MetaProperty")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("meta", &self.meta)?;
        map.serialize_entry("property", &self.property)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type MetaProperty = ({\n\ttype: 'MetaProperty';\n\tmeta: IdentifierName;\n\tproperty: IdentifierName;\n}) & Span;";

impl<'a> Serialize for SpreadElement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "SpreadElement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type SpreadElement = ({\n\ttype: 'SpreadElement';\n\targument: Expression;\n}) & Span;";

impl<'a> Serialize for Argument<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Argument = SpreadElement | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for UpdateExpression<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type UpdateExpression = ({\n\ttype: 'UpdateExpression';\n\toperator: UpdateOperator;\n\tprefix: boolean;\n\targument: SimpleAssignmentTarget;\n}) & Span;";

impl<'a> Serialize for UnaryExpression<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "UnaryExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("operator", &self.operator)?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type UnaryExpression = ({\n\ttype: 'UnaryExpression';\n\toperator: UnaryOperator;\n\targument: Expression;\n}) & Span;";

impl<'a> Serialize for BinaryExpression<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type BinaryExpression = ({\n\ttype: 'BinaryExpression';\n\tleft: Expression;\n\toperator: BinaryOperator;\n\tright: Expression;\n}) & Span;";

impl<'a> Serialize for PrivateInExpression<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type PrivateInExpression = ({\n\ttype: 'PrivateInExpression';\n\tleft: PrivateIdentifier;\n\toperator: BinaryOperator;\n\tright: Expression;\n}) & Span;";

impl<'a> Serialize for LogicalExpression<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type LogicalExpression = ({\n\ttype: 'LogicalExpression';\n\tleft: Expression;\n\toperator: LogicalOperator;\n\tright: Expression;\n}) & Span;";

impl<'a> Serialize for ConditionalExpression<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ConditionalExpression = ({\n\ttype: 'ConditionalExpression';\n\ttest: Expression;\n\tconsequent: Expression;\n\talternate: Expression;\n}) & Span;";

impl<'a> Serialize for AssignmentExpression<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type AssignmentExpression = ({\n\ttype: 'AssignmentExpression';\n\toperator: AssignmentOperator;\n\tleft: AssignmentTarget;\n\tright: Expression;\n}) & Span;";

impl<'a> Serialize for AssignmentTarget<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type AssignmentTarget = IdentifierReference | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression | ArrayAssignmentTarget | ObjectAssignmentTarget;";

impl<'a> Serialize for SimpleAssignmentTarget<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type SimpleAssignmentTarget = IdentifierReference | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for AssignmentTargetPattern<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type AssignmentTargetPattern = ArrayAssignmentTarget | ObjectAssignmentTarget;";

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ArrayAssignmentTarget = ({\n\ttype: 'ArrayAssignmentTarget';\n\telements: Array<AssignmentTargetMaybeDefault | AssignmentTargetRest | null>;\n}) & Span;";

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ObjectAssignmentTarget = ({\n\ttype: 'ObjectAssignmentTarget';\n\tproperties: Array<AssignmentTargetProperty | AssignmentTargetRest>;\n}) & Span;";

impl<'a> Serialize for AssignmentTargetRest<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "RestElement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("argument", &self.target)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type AssignmentTargetRest = ({\n\ttype: 'RestElement';\n\targument: AssignmentTarget;\n}) & Span;";

impl<'a> Serialize for AssignmentTargetMaybeDefault<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type AssignmentTargetMaybeDefault = AssignmentTargetWithDefault | IdentifierReference | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression | ArrayAssignmentTarget | ObjectAssignmentTarget;";

impl<'a> Serialize for AssignmentTargetWithDefault<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AssignmentTargetWithDefault")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("binding", &self.binding)?;
        map.serialize_entry("init", &self.init)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type AssignmentTargetWithDefault = ({\n\ttype: 'AssignmentTargetWithDefault';\n\tbinding: AssignmentTarget;\n\tinit: Expression;\n}) & Span;";

impl<'a> Serialize for AssignmentTargetProperty<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type AssignmentTargetProperty = AssignmentTargetPropertyIdentifier | AssignmentTargetPropertyProperty;";

impl<'a> Serialize for AssignmentTargetPropertyIdentifier<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AssignmentTargetPropertyIdentifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("binding", &self.binding)?;
        map.serialize_entry("init", &self.init)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type AssignmentTargetPropertyIdentifier = ({\n\ttype: 'AssignmentTargetPropertyIdentifier';\n\tbinding: IdentifierReference;\n\tinit: (Expression) | null;\n}) & Span;";

impl<'a> Serialize for AssignmentTargetPropertyProperty<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AssignmentTargetPropertyProperty")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("binding", &self.binding)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type AssignmentTargetPropertyProperty = ({\n\ttype: 'AssignmentTargetPropertyProperty';\n\tname: PropertyKey;\n\tbinding: AssignmentTargetMaybeDefault;\n}) & Span;";

impl<'a> Serialize for SequenceExpression<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "SequenceExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expressions", &self.expressions)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type SequenceExpression = ({\n\ttype: 'SequenceExpression';\n\texpressions: Array<Expression>;\n}) & Span;";

impl Serialize for Super {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Super")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Super = ({\n\ttype: 'Super';\n}) & Span;";

impl<'a> Serialize for AwaitExpression<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AwaitExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type AwaitExpression = ({\n\ttype: 'AwaitExpression';\n\targument: Expression;\n}) & Span;";

impl<'a> Serialize for ChainExpression<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ChainExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ChainExpression = ({\n\ttype: 'ChainExpression';\n\texpression: ChainElement;\n}) & Span;";

impl<'a> Serialize for ChainElement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ChainElement::CallExpression(x) => Serialize::serialize(x, serializer),
            ChainElement::ComputedMemberExpression(x) => Serialize::serialize(x, serializer),
            ChainElement::StaticMemberExpression(x) => Serialize::serialize(x, serializer),
            ChainElement::PrivateFieldExpression(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ChainElement = CallExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for ParenthesizedExpression<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ParenthesizedExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ParenthesizedExpression = ({\n\ttype: 'ParenthesizedExpression';\n\texpression: Expression;\n}) & Span;";

impl<'a> Serialize for Statement<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Statement = BlockStatement | BreakStatement | ContinueStatement | DebuggerStatement | DoWhileStatement | EmptyStatement | ExpressionStatement | ForInStatement | ForOfStatement | ForStatement | IfStatement | LabeledStatement | ReturnStatement | SwitchStatement | ThrowStatement | TryStatement | WhileStatement | WithStatement | VariableDeclaration | Function | Class | TSTypeAliasDeclaration | TSInterfaceDeclaration | TSEnumDeclaration | TSModuleDeclaration | TSImportEqualsDeclaration | ImportDeclaration | ExportAllDeclaration | ExportDefaultDeclaration | ExportNamedDeclaration | TSExportAssignment | TSNamespaceExportDeclaration;";

impl<'a> Serialize for Directive<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Directive")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("directive", &self.directive)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Directive = ({\n\ttype: 'Directive';\n\texpression: StringLiteral;\n\tdirective: string;\n}) & Span;";

impl<'a> Serialize for Hashbang<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Hashbang")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type Hashbang = ({\n\ttype: 'Hashbang';\n\tvalue: string;\n}) & Span;";

impl<'a> Serialize for BlockStatement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BlockStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type BlockStatement = ({\n\ttype: 'BlockStatement';\n\tbody: Array<Statement>;\n}) & Span;";

impl<'a> Serialize for Declaration<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Declaration = VariableDeclaration | Function | Class | TSTypeAliasDeclaration | TSInterfaceDeclaration | TSEnumDeclaration | TSModuleDeclaration | TSImportEqualsDeclaration;";

impl<'a> Serialize for VariableDeclaration<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type VariableDeclaration = ({\n\ttype: 'VariableDeclaration';\n\tkind: VariableDeclarationKind;\n\tdeclarations: Array<VariableDeclarator>;\n\tdeclare: boolean;\n}) & Span;";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type VariableDeclarationKind = 'var' | 'const' | 'let' | 'using' | 'await using';";

impl<'a> Serialize for VariableDeclarator<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type VariableDeclarator = ({\n\ttype: 'VariableDeclarator';\n\tid: BindingPattern;\n\tinit: (Expression) | null;\n\tdefinite: boolean;\n}) & Span;";

impl Serialize for EmptyStatement {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "EmptyStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type EmptyStatement = ({\n\ttype: 'EmptyStatement';\n}) & Span;";

impl<'a> Serialize for ExpressionStatement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ExpressionStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ExpressionStatement = ({\n\ttype: 'ExpressionStatement';\n\texpression: Expression;\n}) & Span;";

impl<'a> Serialize for IfStatement<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type IfStatement = ({\n\ttype: 'IfStatement';\n\ttest: Expression;\n\tconsequent: Statement;\n\talternate: (Statement) | null;\n}) & Span;";

impl<'a> Serialize for DoWhileStatement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "DoWhileStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.serialize_entry("test", &self.test)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type DoWhileStatement = ({\n\ttype: 'DoWhileStatement';\n\tbody: Statement;\n\ttest: Expression;\n}) & Span;";

impl<'a> Serialize for WhileStatement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "WhileStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("test", &self.test)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type WhileStatement = ({\n\ttype: 'WhileStatement';\n\ttest: Expression;\n\tbody: Statement;\n}) & Span;";

impl<'a> Serialize for ForStatement<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ForStatement = ({\n\ttype: 'ForStatement';\n\tinit: (ForStatementInit) | null;\n\ttest: (Expression) | null;\n\tupdate: (Expression) | null;\n\tbody: Statement;\n}) & Span;";

impl<'a> Serialize for ForStatementInit<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ForStatementInit = VariableDeclaration | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for ForInStatement<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ForInStatement = ({\n\ttype: 'ForInStatement';\n\tleft: ForStatementLeft;\n\tright: Expression;\n\tbody: Statement;\n}) & Span;";

impl<'a> Serialize for ForStatementLeft<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ForStatementLeft = VariableDeclaration | IdentifierReference | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression | ArrayAssignmentTarget | ObjectAssignmentTarget;";

impl<'a> Serialize for ForOfStatement<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ForOfStatement = ({\n\ttype: 'ForOfStatement';\n\tawait: boolean;\n\tleft: ForStatementLeft;\n\tright: Expression;\n\tbody: Statement;\n}) & Span;";

impl<'a> Serialize for ContinueStatement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ContinueStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("label", &self.label)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ContinueStatement = ({\n\ttype: 'ContinueStatement';\n\tlabel: (LabelIdentifier) | null;\n}) & Span;";

impl<'a> Serialize for BreakStatement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BreakStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("label", &self.label)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type BreakStatement = ({\n\ttype: 'BreakStatement';\n\tlabel: (LabelIdentifier) | null;\n}) & Span;";

impl<'a> Serialize for ReturnStatement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ReturnStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ReturnStatement = ({\n\ttype: 'ReturnStatement';\n\targument: (Expression) | null;\n}) & Span;";

impl<'a> Serialize for WithStatement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "WithStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("object", &self.object)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type WithStatement = ({\n\ttype: 'WithStatement';\n\tobject: Expression;\n\tbody: Statement;\n}) & Span;";

impl<'a> Serialize for SwitchStatement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "SwitchStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("discriminant", &self.discriminant)?;
        map.serialize_entry("cases", &self.cases)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type SwitchStatement = ({\n\ttype: 'SwitchStatement';\n\tdiscriminant: Expression;\n\tcases: Array<SwitchCase>;\n}) & Span;";

impl<'a> Serialize for SwitchCase<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "SwitchCase")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("test", &self.test)?;
        map.serialize_entry("consequent", &self.consequent)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type SwitchCase = ({\n\ttype: 'SwitchCase';\n\ttest: (Expression) | null;\n\tconsequent: Array<Statement>;\n}) & Span;";

impl<'a> Serialize for LabeledStatement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "LabeledStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("label", &self.label)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type LabeledStatement = ({\n\ttype: 'LabeledStatement';\n\tlabel: LabelIdentifier;\n\tbody: Statement;\n}) & Span;";

impl<'a> Serialize for ThrowStatement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ThrowStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ThrowStatement = ({\n\ttype: 'ThrowStatement';\n\targument: Expression;\n}) & Span;";

impl<'a> Serialize for TryStatement<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TryStatement = ({\n\ttype: 'TryStatement';\n\tblock: BlockStatement;\n\thandler: (CatchClause) | null;\n\tfinalizer: (BlockStatement) | null;\n}) & Span;";

impl<'a> Serialize for CatchClause<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CatchClause")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("param", &self.param)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type CatchClause = ({\n\ttype: 'CatchClause';\n\tparam: (CatchParameter) | null;\n\tbody: BlockStatement;\n}) & Span;";

impl<'a> Serialize for CatchParameter<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CatchParameter")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("pattern", &self.pattern)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type CatchParameter = ({\n\ttype: 'CatchParameter';\n\tpattern: BindingPattern;\n}) & Span;";

impl Serialize for DebuggerStatement {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "DebuggerStatement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type DebuggerStatement = ({\n\ttype: 'DebuggerStatement';\n}) & Span;";

impl<'a> Serialize for BindingPattern<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        self.kind.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("optional", &self.optional)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type BindingPattern = ({\n\ttypeAnnotation: (TSTypeAnnotation) | null;\n\toptional: boolean;\n}) & (BindingIdentifier | ObjectPattern | ArrayPattern | AssignmentPattern);";

impl<'a> Serialize for BindingPatternKind<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            BindingPatternKind::BindingIdentifier(x) => Serialize::serialize(x, serializer),
            BindingPatternKind::ObjectPattern(x) => Serialize::serialize(x, serializer),
            BindingPatternKind::ArrayPattern(x) => Serialize::serialize(x, serializer),
            BindingPatternKind::AssignmentPattern(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type BindingPatternKind = BindingIdentifier | ObjectPattern | ArrayPattern | AssignmentPattern;";

impl<'a> Serialize for AssignmentPattern<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "AssignmentPattern")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("left", &self.left)?;
        map.serialize_entry("right", &self.right)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type AssignmentPattern = ({\n\ttype: 'AssignmentPattern';\n\tleft: BindingPattern;\n\tright: Expression;\n}) & Span;";

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ObjectPattern = ({\n\ttype: 'ObjectPattern';\n\tproperties: Array<BindingProperty | BindingRestElement>;\n}) & Span;";

impl<'a> Serialize for BindingProperty<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type BindingProperty = ({\n\ttype: 'BindingProperty';\n\tkey: PropertyKey;\n\tvalue: BindingPattern;\n\tshorthand: boolean;\n\tcomputed: boolean;\n}) & Span;";

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ArrayPattern = ({\n\ttype: 'ArrayPattern';\n\telements: Array<BindingPattern | BindingRestElement | null>;\n}) & Span;";

impl<'a> Serialize for BindingRestElement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "RestElement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type BindingRestElement = ({\n\ttype: 'RestElement';\n\targument: BindingPattern;\n}) & Span;";

impl<'a> Serialize for Function<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", &self.r#type)?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Function = ({\n\ttype: FunctionType;\n\tid: (BindingIdentifier) | null;\n\tgenerator: boolean;\n\tasync: boolean;\n\tdeclare: boolean;\n\ttypeParameters: (TSTypeParameterDeclaration) | null;\n\tthisParam: (TSThisParameter) | null;\n\tparams: FormalParameters;\n\treturnType: (TSTypeAnnotation) | null;\n\tbody: (FunctionBody) | null;\n}) & Span;";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type FunctionType = 'FunctionDeclaration' | 'FunctionExpression' | 'TSDeclareFunction' | 'TSEmptyBodyFunctionExpression';";

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type FormalParameters = ({\n\ttype: 'FormalParameters';\n\tkind: FormalParameterKind;\n\titems: Array<FormalParameter | FormalParameterRest>;\n}) & Span;";

impl<'a> Serialize for FormalParameter<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type FormalParameter = ({\n\ttype: 'FormalParameter';\n\tdecorators: Array<Decorator>;\n\tpattern: BindingPattern;\n\taccessibility: (TSAccessibility) | null;\n\treadonly: boolean;\n\toverride: boolean;\n}) & Span;";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type FormalParameterKind = 'FormalParameter' | 'UniqueFormalParameters' | 'ArrowFormalParameters' | 'Signature';";

impl<'a> Serialize for FunctionBody<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "FunctionBody")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("directives", &self.directives)?;
        map.serialize_entry("statements", &self.statements)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type FunctionBody = ({\n\ttype: 'FunctionBody';\n\tdirectives: Array<Directive>;\n\tstatements: Array<Statement>;\n}) & Span;";

impl<'a> Serialize for ArrowFunctionExpression<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ArrowFunctionExpression = ({\n\ttype: 'ArrowFunctionExpression';\n\texpression: boolean;\n\tasync: boolean;\n\ttypeParameters: (TSTypeParameterDeclaration) | null;\n\tparams: FormalParameters;\n\treturnType: (TSTypeAnnotation) | null;\n\tbody: FunctionBody;\n}) & Span;";

impl<'a> Serialize for YieldExpression<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "YieldExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("delegate", &self.delegate)?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type YieldExpression = ({\n\ttype: 'YieldExpression';\n\tdelegate: boolean;\n\targument: (Expression) | null;\n}) & Span;";

impl<'a> Serialize for Class<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", &self.r#type)?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Class = ({\n\ttype: ClassType;\n\tdecorators: Array<Decorator>;\n\tid: (BindingIdentifier) | null;\n\ttypeParameters: (TSTypeParameterDeclaration) | null;\n\tsuperClass: (Expression) | null;\n\tsuperTypeParameters: (TSTypeParameterInstantiation) | null;\n\timplements: (Array<TSClassImplements>) | null;\n\tbody: ClassBody;\n\tabstract: boolean;\n\tdeclare: boolean;\n}) & Span;";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type ClassType = 'ClassDeclaration' | 'ClassExpression';";

impl<'a> Serialize for ClassBody<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ClassBody")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type ClassBody = ({\n\ttype: 'ClassBody';\n\tbody: Array<ClassElement>;\n}) & Span;";

impl<'a> Serialize for ClassElement<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ClassElement = StaticBlock | MethodDefinition | PropertyDefinition | AccessorProperty | TSIndexSignature;";

impl<'a> Serialize for MethodDefinition<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", &self.r#type)?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type MethodDefinition = ({\n\ttype: MethodDefinitionType;\n\tdecorators: Array<Decorator>;\n\tkey: PropertyKey;\n\tvalue: Function;\n\tkind: MethodDefinitionKind;\n\tcomputed: boolean;\n\tstatic: boolean;\n\toverride: boolean;\n\toptional: boolean;\n\taccessibility: (TSAccessibility) | null;\n}) & Span;";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type MethodDefinitionType = 'MethodDefinition' | 'TSAbstractMethodDefinition';";

impl<'a> Serialize for PropertyDefinition<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", &self.r#type)?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type PropertyDefinition = ({\n\ttype: PropertyDefinitionType;\n\tdecorators: Array<Decorator>;\n\tkey: PropertyKey;\n\tvalue: (Expression) | null;\n\tcomputed: boolean;\n\tstatic: boolean;\n\tdeclare: boolean;\n\toverride: boolean;\n\toptional: boolean;\n\tdefinite: boolean;\n\treadonly: boolean;\n\ttypeAnnotation: (TSTypeAnnotation) | null;\n\taccessibility: (TSAccessibility) | null;\n}) & Span;";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type PropertyDefinitionType = 'PropertyDefinition' | 'TSAbstractPropertyDefinition';";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type MethodDefinitionKind = 'constructor' | 'method' | 'get' | 'set';";

impl<'a> Serialize for PrivateIdentifier<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "PrivateIdentifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type PrivateIdentifier = ({\n\ttype: 'PrivateIdentifier';\n\tname: string;\n}) & Span;";

impl<'a> Serialize for StaticBlock<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "StaticBlock")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type StaticBlock = ({\n\ttype: 'StaticBlock';\n\tbody: Array<Statement>;\n}) & Span;";

impl<'a> Serialize for ModuleDeclaration<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ModuleDeclaration = ImportDeclaration | ExportAllDeclaration | ExportDefaultDeclaration | ExportNamedDeclaration | TSExportAssignment | TSNamespaceExportDeclaration;";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type AccessorPropertyType = 'AccessorProperty' | 'TSAbstractAccessorProperty';";

impl<'a> Serialize for AccessorProperty<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", &self.r#type)?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type AccessorProperty = ({\n\ttype: AccessorPropertyType;\n\tdecorators: Array<Decorator>;\n\tkey: PropertyKey;\n\tvalue: (Expression) | null;\n\tcomputed: boolean;\n\tstatic: boolean;\n\tdefinite: boolean;\n\ttypeAnnotation: (TSTypeAnnotation) | null;\n\taccessibility: (TSAccessibility) | null;\n}) & Span;";

impl<'a> Serialize for ImportExpression<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ImportExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("source", &self.source)?;
        map.serialize_entry("arguments", &self.arguments)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ImportExpression = ({\n\ttype: 'ImportExpression';\n\tsource: Expression;\n\targuments: Array<Expression>;\n}) & Span;";

impl<'a> Serialize for ImportDeclaration<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ImportDeclaration")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("specifiers", &self.specifiers)?;
        map.serialize_entry("source", &self.source)?;
        map.serialize_entry("withClause", &self.with_clause)?;
        map.serialize_entry("importKind", &self.import_kind)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ImportDeclaration = ({\n\ttype: 'ImportDeclaration';\n\tspecifiers: (Array<ImportDeclarationSpecifier>) | null;\n\tsource: StringLiteral;\n\twithClause: (WithClause) | null;\n\timportKind: ImportOrExportKind;\n}) & Span;";

impl<'a> Serialize for ImportDeclarationSpecifier<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ImportDeclarationSpecifier = ImportSpecifier | ImportDefaultSpecifier | ImportNamespaceSpecifier;";

impl<'a> Serialize for ImportSpecifier<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ImportSpecifier = ({\n\ttype: 'ImportSpecifier';\n\timported: ModuleExportName;\n\tlocal: BindingIdentifier;\n\timportKind: ImportOrExportKind;\n}) & Span;";

impl<'a> Serialize for ImportDefaultSpecifier<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ImportDefaultSpecifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("local", &self.local)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ImportDefaultSpecifier = ({\n\ttype: 'ImportDefaultSpecifier';\n\tlocal: BindingIdentifier;\n}) & Span;";

impl<'a> Serialize for ImportNamespaceSpecifier<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ImportNamespaceSpecifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("local", &self.local)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ImportNamespaceSpecifier = ({\n\ttype: 'ImportNamespaceSpecifier';\n\tlocal: BindingIdentifier;\n}) & Span;";

impl<'a> Serialize for WithClause<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "WithClause")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("attributesKeyword", &self.attributes_keyword)?;
        map.serialize_entry("withEntries", &self.with_entries)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type WithClause = ({\n\ttype: 'WithClause';\n\tattributesKeyword: IdentifierName;\n\twithEntries: Array<ImportAttribute>;\n}) & Span;";

impl<'a> Serialize for ImportAttribute<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ImportAttribute")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("key", &self.key)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ImportAttribute = ({\n\ttype: 'ImportAttribute';\n\tkey: ImportAttributeKey;\n\tvalue: StringLiteral;\n}) & Span;";

impl<'a> Serialize for ImportAttributeKey<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ImportAttributeKey::Identifier(x) => Serialize::serialize(x, serializer),
            ImportAttributeKey::StringLiteral(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type ImportAttributeKey = IdentifierName | StringLiteral;";

impl<'a> Serialize for ExportNamedDeclaration<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ExportNamedDeclaration = ({\n\ttype: 'ExportNamedDeclaration';\n\tdeclaration: (Declaration) | null;\n\tspecifiers: Array<ExportSpecifier>;\n\tsource: (StringLiteral) | null;\n\texportKind: ImportOrExportKind;\n\twithClause: (WithClause) | null;\n}) & Span;";

impl<'a> Serialize for ExportDefaultDeclaration<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ExportDefaultDeclaration")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("declaration", &self.declaration)?;
        map.serialize_entry("exported", &self.exported)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ExportDefaultDeclaration = ({\n\ttype: 'ExportDefaultDeclaration';\n\tdeclaration: ExportDefaultDeclarationKind;\n\texported: ModuleExportName;\n}) & Span;";

impl<'a> Serialize for ExportAllDeclaration<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ExportAllDeclaration = ({\n\ttype: 'ExportAllDeclaration';\n\texported: (ModuleExportName) | null;\n\tsource: StringLiteral;\n\twithClause: (WithClause) | null;\n\texportKind: ImportOrExportKind;\n}) & Span;";

impl<'a> Serialize for ExportSpecifier<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ExportSpecifier = ({\n\ttype: 'ExportSpecifier';\n\tlocal: ModuleExportName;\n\texported: ModuleExportName;\n\texportKind: ImportOrExportKind;\n}) & Span;";

impl<'a> Serialize for ExportDefaultDeclarationKind<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ExportDefaultDeclarationKind = Function | Class | TSInterfaceDeclaration | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for ModuleExportName<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ModuleExportName::IdentifierName(x) => Serialize::serialize(x, serializer),
            ModuleExportName::IdentifierReference(x) => Serialize::serialize(x, serializer),
            ModuleExportName::StringLiteral(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type ModuleExportName = IdentifierName | IdentifierReference | StringLiteral;";

impl<'a> Serialize for TSThisParameter<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSThisParameter")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("thisSpan", &self.this_span)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSThisParameter = ({\n\ttype: 'TSThisParameter';\n\tthisSpan: Span;\n\ttypeAnnotation: (TSTypeAnnotation) | null;\n}) & Span;";

impl<'a> Serialize for TSEnumDeclaration<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSEnumDeclaration = ({\n\ttype: 'TSEnumDeclaration';\n\tid: BindingIdentifier;\n\tmembers: Array<TSEnumMember>;\n\tconst: boolean;\n\tdeclare: boolean;\n}) & Span;";

impl<'a> Serialize for TSEnumMember<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSEnumMember")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("initializer", &self.initializer)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSEnumMember = ({\n\ttype: 'TSEnumMember';\n\tid: TSEnumMemberName;\n\tinitializer: (Expression) | null;\n}) & Span;";

impl<'a> Serialize for TSEnumMemberName<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSEnumMemberName::StaticIdentifier(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::StaticStringLiteral(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::StaticTemplateLiteral(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::StaticNumericLiteral(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::BooleanLiteral(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::NullLiteral(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::NumericLiteral(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::BigIntLiteral(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::RegExpLiteral(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::StringLiteral(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::TemplateLiteral(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::Identifier(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::MetaProperty(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::Super(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ArrayExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ArrowFunctionExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::AssignmentExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::AwaitExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::BinaryExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::CallExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ChainExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ClassExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ConditionalExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::FunctionExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ImportExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::LogicalExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::NewExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ObjectExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ParenthesizedExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::SequenceExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::TaggedTemplateExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ThisExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::UnaryExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::UpdateExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::YieldExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::PrivateInExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::JSXElement(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::JSXFragment(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::TSAsExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::TSSatisfiesExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::TSTypeAssertion(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::TSNonNullExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::TSInstantiationExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ComputedMemberExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::StaticMemberExpression(x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::PrivateFieldExpression(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSEnumMemberName = IdentifierName | StringLiteral | TemplateLiteral | NumericLiteral | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for TSTypeAnnotation<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeAnnotation")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSTypeAnnotation = ({\n\ttype: 'TSTypeAnnotation';\n\ttypeAnnotation: TSType;\n}) & Span;";

impl<'a> Serialize for TSLiteralType<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSLiteralType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("literal", &self.literal)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSLiteralType = ({\n\ttype: 'TSLiteralType';\n\tliteral: TSLiteral;\n}) & Span;";

impl<'a> Serialize for TSLiteral<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSLiteral = BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | UnaryExpression;";

impl<'a> Serialize for TSType<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSType = TSAnyKeyword | TSBigIntKeyword | TSBooleanKeyword | TSIntrinsicKeyword | TSNeverKeyword | TSNullKeyword | TSNumberKeyword | TSObjectKeyword | TSStringKeyword | TSSymbolKeyword | TSUndefinedKeyword | TSUnknownKeyword | TSVoidKeyword | TSArrayType | TSConditionalType | TSConstructorType | TSFunctionType | TSImportType | TSIndexedAccessType | TSInferType | TSIntersectionType | TSLiteralType | TSMappedType | TSNamedTupleMember | TSQualifiedName | TSTemplateLiteralType | TSThisType | TSTupleType | TSTypeLiteral | TSTypeOperator | TSTypePredicate | TSTypeQuery | TSTypeReference | TSUnionType | TSParenthesizedType | JSDocNullableType | JSDocNonNullableType | JSDocUnknownType;";

impl<'a> Serialize for TSConditionalType<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSConditionalType = ({\n\ttype: 'TSConditionalType';\n\tcheckType: TSType;\n\textendsType: TSType;\n\ttrueType: TSType;\n\tfalseType: TSType;\n}) & Span;";

impl<'a> Serialize for TSUnionType<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSUnionType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("types", &self.types)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSUnionType = ({\n\ttype: 'TSUnionType';\n\ttypes: Array<TSType>;\n}) & Span;";

impl<'a> Serialize for TSIntersectionType<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSIntersectionType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("types", &self.types)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSIntersectionType = ({\n\ttype: 'TSIntersectionType';\n\ttypes: Array<TSType>;\n}) & Span;";

impl<'a> Serialize for TSParenthesizedType<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSParenthesizedType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSParenthesizedType = ({\n\ttype: 'TSParenthesizedType';\n\ttypeAnnotation: TSType;\n}) & Span;";

impl<'a> Serialize for TSTypeOperator<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeOperator")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("operator", &self.operator)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSTypeOperator = ({\n\ttype: 'TSTypeOperator';\n\toperator: TSTypeOperatorOperator;\n\ttypeAnnotation: TSType;\n}) & Span;";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSTypeOperatorOperator = 'keyof' | 'unique' | 'readonly';";

impl<'a> Serialize for TSArrayType<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSArrayType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("elementType", &self.element_type)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSArrayType = ({\n\ttype: 'TSArrayType';\n\telementType: TSType;\n}) & Span;";

impl<'a> Serialize for TSIndexedAccessType<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSIndexedAccessType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("objectType", &self.object_type)?;
        map.serialize_entry("indexType", &self.index_type)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSIndexedAccessType = ({\n\ttype: 'TSIndexedAccessType';\n\tobjectType: TSType;\n\tindexType: TSType;\n}) & Span;";

impl<'a> Serialize for TSTupleType<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTupleType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("elementTypes", &self.element_types)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSTupleType = ({\n\ttype: 'TSTupleType';\n\telementTypes: Array<TSTupleElement>;\n}) & Span;";

impl<'a> Serialize for TSNamedTupleMember<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSNamedTupleMember = ({\n\ttype: 'TSNamedTupleMember';\n\telementType: TSTupleElement;\n\tlabel: IdentifierName;\n\toptional: boolean;\n}) & Span;";

impl<'a> Serialize for TSOptionalType<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSOptionalType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSOptionalType = ({\n\ttype: 'TSOptionalType';\n\ttypeAnnotation: TSType;\n}) & Span;";

impl<'a> Serialize for TSRestType<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSRestType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSRestType = ({\n\ttype: 'TSRestType';\n\ttypeAnnotation: TSType;\n}) & Span;";

impl<'a> Serialize for TSTupleElement<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSTupleElement = TSOptionalType | TSRestType | TSAnyKeyword | TSBigIntKeyword | TSBooleanKeyword | TSIntrinsicKeyword | TSNeverKeyword | TSNullKeyword | TSNumberKeyword | TSObjectKeyword | TSStringKeyword | TSSymbolKeyword | TSUndefinedKeyword | TSUnknownKeyword | TSVoidKeyword | TSArrayType | TSConditionalType | TSConstructorType | TSFunctionType | TSImportType | TSIndexedAccessType | TSInferType | TSIntersectionType | TSLiteralType | TSMappedType | TSNamedTupleMember | TSQualifiedName | TSTemplateLiteralType | TSThisType | TSTupleType | TSTypeLiteral | TSTypeOperator | TSTypePredicate | TSTypeQuery | TSTypeReference | TSUnionType | TSParenthesizedType | JSDocNullableType | JSDocNonNullableType | JSDocUnknownType;";

impl Serialize for TSAnyKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSAnyKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSAnyKeyword = ({\n\ttype: 'TSAnyKeyword';\n}) & Span;";

impl Serialize for TSStringKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSStringKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSStringKeyword = ({\n\ttype: 'TSStringKeyword';\n}) & Span;";

impl Serialize for TSBooleanKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSBooleanKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSBooleanKeyword = ({\n\ttype: 'TSBooleanKeyword';\n}) & Span;";

impl Serialize for TSNumberKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNumberKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSNumberKeyword = ({\n\ttype: 'TSNumberKeyword';\n}) & Span;";

impl Serialize for TSNeverKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNeverKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSNeverKeyword = ({\n\ttype: 'TSNeverKeyword';\n}) & Span;";

impl Serialize for TSIntrinsicKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSIntrinsicKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSIntrinsicKeyword = ({\n\ttype: 'TSIntrinsicKeyword';\n}) & Span;";

impl Serialize for TSUnknownKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSUnknownKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSUnknownKeyword = ({\n\ttype: 'TSUnknownKeyword';\n}) & Span;";

impl Serialize for TSNullKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNullKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSNullKeyword = ({\n\ttype: 'TSNullKeyword';\n}) & Span;";

impl Serialize for TSUndefinedKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSUndefinedKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSUndefinedKeyword = ({\n\ttype: 'TSUndefinedKeyword';\n}) & Span;";

impl Serialize for TSVoidKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSVoidKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSVoidKeyword = ({\n\ttype: 'TSVoidKeyword';\n}) & Span;";

impl Serialize for TSSymbolKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSSymbolKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSSymbolKeyword = ({\n\ttype: 'TSSymbolKeyword';\n}) & Span;";

impl Serialize for TSThisType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSThisType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSThisType = ({\n\ttype: 'TSThisType';\n}) & Span;";

impl Serialize for TSObjectKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSObjectKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSObjectKeyword = ({\n\ttype: 'TSObjectKeyword';\n}) & Span;";

impl Serialize for TSBigIntKeyword {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSBigIntKeyword")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSBigIntKeyword = ({\n\ttype: 'TSBigIntKeyword';\n}) & Span;";

impl<'a> Serialize for TSTypeReference<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeReference")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeName", &self.type_name)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSTypeReference = ({\n\ttype: 'TSTypeReference';\n\ttypeName: TSTypeName;\n\ttypeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

impl<'a> Serialize for TSTypeName<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSTypeName::IdentifierReference(x) => Serialize::serialize(x, serializer),
            TSTypeName::QualifiedName(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSTypeName = IdentifierReference | TSQualifiedName;";

impl<'a> Serialize for TSQualifiedName<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSQualifiedName")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("left", &self.left)?;
        map.serialize_entry("right", &self.right)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSQualifiedName = ({\n\ttype: 'TSQualifiedName';\n\tleft: TSTypeName;\n\tright: IdentifierName;\n}) & Span;";

impl<'a> Serialize for TSTypeParameterInstantiation<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeParameterInstantiation")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("params", &self.params)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSTypeParameterInstantiation = ({\n\ttype: 'TSTypeParameterInstantiation';\n\tparams: Array<TSType>;\n}) & Span;";

impl<'a> Serialize for TSTypeParameter<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSTypeParameter = ({\n\ttype: 'TSTypeParameter';\n\tname: BindingIdentifier;\n\tconstraint: (TSType) | null;\n\tdefault: (TSType) | null;\n\tin: boolean;\n\tout: boolean;\n\tconst: boolean;\n}) & Span;";

impl<'a> Serialize for TSTypeParameterDeclaration<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeParameterDeclaration")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("params", &self.params)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSTypeParameterDeclaration = ({\n\ttype: 'TSTypeParameterDeclaration';\n\tparams: Array<TSTypeParameter>;\n}) & Span;";

impl<'a> Serialize for TSTypeAliasDeclaration<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSTypeAliasDeclaration = ({\n\ttype: 'TSTypeAliasDeclaration';\n\tid: BindingIdentifier;\n\ttypeParameters: (TSTypeParameterDeclaration) | null;\n\ttypeAnnotation: TSType;\n\tdeclare: boolean;\n}) & Span;";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSAccessibility = 'private' | 'protected' | 'public';";

impl<'a> Serialize for TSClassImplements<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSClassImplements")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSClassImplements = ({\n\ttype: 'TSClassImplements';\n\texpression: TSTypeName;\n\ttypeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

impl<'a> Serialize for TSInterfaceDeclaration<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSInterfaceDeclaration = ({\n\ttype: 'TSInterfaceDeclaration';\n\tid: BindingIdentifier;\n\textends: (Array<TSInterfaceHeritage>) | null;\n\ttypeParameters: (TSTypeParameterDeclaration) | null;\n\tbody: TSInterfaceBody;\n\tdeclare: boolean;\n}) & Span;";

impl<'a> Serialize for TSInterfaceBody<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSInterfaceBody")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSInterfaceBody = ({\n\ttype: 'TSInterfaceBody';\n\tbody: Array<TSSignature>;\n}) & Span;";

impl<'a> Serialize for TSPropertySignature<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSPropertySignature = ({\n\ttype: 'TSPropertySignature';\n\tcomputed: boolean;\n\toptional: boolean;\n\treadonly: boolean;\n\tkey: PropertyKey;\n\ttypeAnnotation: (TSTypeAnnotation) | null;\n}) & Span;";

impl<'a> Serialize for TSSignature<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSSignature = TSIndexSignature | TSPropertySignature | TSCallSignatureDeclaration | TSConstructSignatureDeclaration | TSMethodSignature;";

impl<'a> Serialize for TSIndexSignature<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSIndexSignature")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("parameters", &self.parameters)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("readonly", &self.readonly)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSIndexSignature = ({\n\ttype: 'TSIndexSignature';\n\tparameters: Array<TSIndexSignatureName>;\n\ttypeAnnotation: TSTypeAnnotation;\n\treadonly: boolean;\n}) & Span;";

impl<'a> Serialize for TSCallSignatureDeclaration<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSCallSignatureDeclaration = ({\n\ttype: 'TSCallSignatureDeclaration';\n\ttypeParameters: (TSTypeParameterDeclaration) | null;\n\tthisParam: (TSThisParameter) | null;\n\tparams: FormalParameters;\n\treturnType: (TSTypeAnnotation) | null;\n}) & Span;";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSMethodSignatureKind = 'method' | 'get' | 'set';";

impl<'a> Serialize for TSMethodSignature<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSMethodSignature = ({\n\ttype: 'TSMethodSignature';\n\tkey: PropertyKey;\n\tcomputed: boolean;\n\toptional: boolean;\n\tkind: TSMethodSignatureKind;\n\ttypeParameters: (TSTypeParameterDeclaration) | null;\n\tthisParam: (TSThisParameter) | null;\n\tparams: FormalParameters;\n\treturnType: (TSTypeAnnotation) | null;\n}) & Span;";

impl<'a> Serialize for TSConstructSignatureDeclaration<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSConstructSignatureDeclaration = ({\n\ttype: 'TSConstructSignatureDeclaration';\n\ttypeParameters: (TSTypeParameterDeclaration) | null;\n\tparams: FormalParameters;\n\treturnType: (TSTypeAnnotation) | null;\n}) & Span;";

impl<'a> Serialize for TSIndexSignatureName<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Identifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSIndexSignatureName = ({\n\ttype: 'Identifier';\n\tname: string;\n\ttypeAnnotation: TSTypeAnnotation;\n}) & Span;";

impl<'a> Serialize for TSInterfaceHeritage<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSInterfaceHeritage")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSInterfaceHeritage = ({\n\ttype: 'TSInterfaceHeritage';\n\texpression: Expression;\n\ttypeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

impl<'a> Serialize for TSTypePredicate<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSTypePredicate = ({\n\ttype: 'TSTypePredicate';\n\tparameterName: TSTypePredicateName;\n\tasserts: boolean;\n\ttypeAnnotation: (TSTypeAnnotation) | null;\n}) & Span;";

impl<'a> Serialize for TSTypePredicateName<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSTypePredicateName::Identifier(x) => Serialize::serialize(x, serializer),
            TSTypePredicateName::This(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSTypePredicateName = IdentifierName | TSThisType;";

impl<'a> Serialize for TSModuleDeclaration<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSModuleDeclaration = ({\n\ttype: 'TSModuleDeclaration';\n\tid: TSModuleDeclarationName;\n\tbody: (TSModuleDeclarationBody) | null;\n\tkind: TSModuleDeclarationKind;\n\tdeclare: boolean;\n}) & Span;";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSModuleDeclarationKind = 'global' | 'module' | 'namespace';";

impl<'a> Serialize for TSModuleDeclarationName<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSModuleDeclarationName::Identifier(x) => Serialize::serialize(x, serializer),
            TSModuleDeclarationName::StringLiteral(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSModuleDeclarationName = BindingIdentifier | StringLiteral;";

impl<'a> Serialize for TSModuleDeclarationBody<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSModuleDeclarationBody::TSModuleDeclaration(x) => Serialize::serialize(x, serializer),
            TSModuleDeclarationBody::TSModuleBlock(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSModuleDeclarationBody = TSModuleDeclaration | TSModuleBlock;";

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSModuleBlock = ({\n\ttype: 'TSModuleBlock';\n\tbody: Array<Statement>;\n}) & Span;";

impl<'a> Serialize for TSTypeLiteral<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeLiteral")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("members", &self.members)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSTypeLiteral = ({\n\ttype: 'TSTypeLiteral';\n\tmembers: Array<TSSignature>;\n}) & Span;";

impl<'a> Serialize for TSInferType<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSInferType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeParameter", &self.type_parameter)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSInferType = ({\n\ttype: 'TSInferType';\n\ttypeParameter: TSTypeParameter;\n}) & Span;";

impl<'a> Serialize for TSTypeQuery<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeQuery")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("exprName", &self.expr_name)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSTypeQuery = ({\n\ttype: 'TSTypeQuery';\n\texprName: TSTypeQueryExprName;\n\ttypeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

impl<'a> Serialize for TSTypeQueryExprName<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSTypeQueryExprName::TSImportType(x) => Serialize::serialize(x, serializer),
            TSTypeQueryExprName::IdentifierReference(x) => Serialize::serialize(x, serializer),
            TSTypeQueryExprName::QualifiedName(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSTypeQueryExprName = TSImportType | IdentifierReference | TSQualifiedName;";

impl<'a> Serialize for TSImportType<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSImportType = ({\n\ttype: 'TSImportType';\n\tisTypeOf: boolean;\n\tparameter: TSType;\n\tqualifier: (TSTypeName) | null;\n\tattributes: (TSImportAttributes) | null;\n\ttypeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

impl<'a> Serialize for TSImportAttributes<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSImportAttributes")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("attributesKeyword", &self.attributes_keyword)?;
        map.serialize_entry("elements", &self.elements)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSImportAttributes = ({\n\ttype: 'TSImportAttributes';\n\tattributesKeyword: IdentifierName;\n\telements: Array<TSImportAttribute>;\n}) & Span;";

impl<'a> Serialize for TSImportAttribute<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSImportAttribute")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSImportAttribute = ({\n\ttype: 'TSImportAttribute';\n\tname: TSImportAttributeName;\n\tvalue: Expression;\n}) & Span;";

impl<'a> Serialize for TSImportAttributeName<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSImportAttributeName::Identifier(x) => Serialize::serialize(x, serializer),
            TSImportAttributeName::StringLiteral(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSImportAttributeName = IdentifierName | StringLiteral;";

impl<'a> Serialize for TSFunctionType<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSFunctionType = ({\n\ttype: 'TSFunctionType';\n\ttypeParameters: (TSTypeParameterDeclaration) | null;\n\tthisParam: (TSThisParameter) | null;\n\tparams: FormalParameters;\n\treturnType: TSTypeAnnotation;\n}) & Span;";

impl<'a> Serialize for TSConstructorType<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSConstructorType = ({\n\ttype: 'TSConstructorType';\n\tabstract: boolean;\n\ttypeParameters: (TSTypeParameterDeclaration) | null;\n\tparams: FormalParameters;\n\treturnType: TSTypeAnnotation;\n}) & Span;";

impl<'a> Serialize for TSMappedType<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSMappedType = ({\n\ttype: 'TSMappedType';\n\ttypeParameter: TSTypeParameter;\n\tnameType: (TSType) | null;\n\ttypeAnnotation: (TSType) | null;\n\toptional: TSMappedTypeModifierOperator;\n\treadonly: TSMappedTypeModifierOperator;\n}) & Span;";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSMappedTypeModifierOperator = 'true' | '+' | '-' | 'none';";

impl<'a> Serialize for TSTemplateLiteralType<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTemplateLiteralType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("quasis", &self.quasis)?;
        map.serialize_entry("types", &self.types)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSTemplateLiteralType = ({\n\ttype: 'TSTemplateLiteralType';\n\tquasis: Array<TemplateElement>;\n\ttypes: Array<TSType>;\n}) & Span;";

impl<'a> Serialize for TSAsExpression<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSAsExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSAsExpression = ({\n\ttype: 'TSAsExpression';\n\texpression: Expression;\n\ttypeAnnotation: TSType;\n}) & Span;";

impl<'a> Serialize for TSSatisfiesExpression<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSSatisfiesExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSSatisfiesExpression = ({\n\ttype: 'TSSatisfiesExpression';\n\texpression: Expression;\n\ttypeAnnotation: TSType;\n}) & Span;";

impl<'a> Serialize for TSTypeAssertion<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSTypeAssertion")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSTypeAssertion = ({\n\ttype: 'TSTypeAssertion';\n\texpression: Expression;\n\ttypeAnnotation: TSType;\n}) & Span;";

impl<'a> Serialize for TSImportEqualsDeclaration<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSImportEqualsDeclaration = ({\n\ttype: 'TSImportEqualsDeclaration';\n\tid: BindingIdentifier;\n\tmoduleReference: TSModuleReference;\n\timportKind: ImportOrExportKind;\n}) & Span;";

impl<'a> Serialize for TSModuleReference<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TSModuleReference::ExternalModuleReference(x) => Serialize::serialize(x, serializer),
            TSModuleReference::IdentifierReference(x) => Serialize::serialize(x, serializer),
            TSModuleReference::QualifiedName(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSModuleReference = TSExternalModuleReference | IdentifierReference | TSQualifiedName;";

impl<'a> Serialize for TSExternalModuleReference<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSExternalModuleReference")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSExternalModuleReference = ({\n\ttype: 'TSExternalModuleReference';\n\texpression: StringLiteral;\n}) & Span;";

impl<'a> Serialize for TSNonNullExpression<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNonNullExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSNonNullExpression = ({\n\ttype: 'TSNonNullExpression';\n\texpression: Expression;\n}) & Span;";

impl<'a> Serialize for Decorator<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Decorator")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type Decorator = ({\n\ttype: 'Decorator';\n\texpression: Expression;\n}) & Span;";

impl<'a> Serialize for TSExportAssignment<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSExportAssignment")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSExportAssignment = ({\n\ttype: 'TSExportAssignment';\n\texpression: Expression;\n}) & Span;";

impl<'a> Serialize for TSNamespaceExportDeclaration<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSNamespaceExportDeclaration")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("id", &self.id)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSNamespaceExportDeclaration = ({\n\ttype: 'TSNamespaceExportDeclaration';\n\tid: IdentifierName;\n}) & Span;";

impl<'a> Serialize for TSInstantiationExpression<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "TSInstantiationExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.serialize_entry("typeParameters", &self.type_parameters)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSInstantiationExpression = ({\n\ttype: 'TSInstantiationExpression';\n\texpression: Expression;\n\ttypeParameters: TSTypeParameterInstantiation;\n}) & Span;";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ImportOrExportKind = 'value' | 'type';";

impl<'a> Serialize for JSDocNullableType<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSDocNullableType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("postfix", &self.postfix)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSDocNullableType = ({\n\ttype: 'JSDocNullableType';\n\ttypeAnnotation: TSType;\n\tpostfix: boolean;\n}) & Span;";

impl<'a> Serialize for JSDocNonNullableType<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSDocNonNullableType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("typeAnnotation", &self.type_annotation)?;
        map.serialize_entry("postfix", &self.postfix)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSDocNonNullableType = ({\n\ttype: 'JSDocNonNullableType';\n\ttypeAnnotation: TSType;\n\tpostfix: boolean;\n}) & Span;";

impl Serialize for JSDocUnknownType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSDocUnknownType")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type JSDocUnknownType = ({\n\ttype: 'JSDocUnknownType';\n}) & Span;";

impl<'a> Serialize for JSXElement<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXElement = ({\n\ttype: 'JSXElement';\n\topeningElement: JSXOpeningElement;\n\tclosingElement: (JSXClosingElement) | null;\n\tchildren: Array<JSXChild>;\n}) & Span;";

impl<'a> Serialize for JSXOpeningElement<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXOpeningElement = ({\n\ttype: 'JSXOpeningElement';\n\tselfClosing: boolean;\n\tname: JSXElementName;\n\tattributes: Array<JSXAttributeItem>;\n\ttypeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

impl<'a> Serialize for JSXClosingElement<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXClosingElement")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXClosingElement = ({\n\ttype: 'JSXClosingElement';\n\tname: JSXElementName;\n}) & Span;";

impl<'a> Serialize for JSXFragment<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXFragment = ({\n\ttype: 'JSXFragment';\n\topeningFragment: JSXOpeningFragment;\n\tclosingFragment: JSXClosingFragment;\n\tchildren: Array<JSXChild>;\n}) & Span;";

impl Serialize for JSXOpeningFragment {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXOpeningFragment")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type JSXOpeningFragment = ({\n\ttype: 'JSXOpeningFragment';\n}) & Span;";

impl Serialize for JSXClosingFragment {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXClosingFragment")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type JSXClosingFragment = ({\n\ttype: 'JSXClosingFragment';\n}) & Span;";

impl<'a> Serialize for JSXNamespacedName<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXNamespacedName")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("namespace", &self.namespace)?;
        map.serialize_entry("property", &self.property)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXNamespacedName = ({\n\ttype: 'JSXNamespacedName';\n\tnamespace: JSXIdentifier;\n\tproperty: JSXIdentifier;\n}) & Span;";

impl<'a> Serialize for JSXMemberExpression<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXMemberExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("object", &self.object)?;
        map.serialize_entry("property", &self.property)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXMemberExpression = ({\n\ttype: 'JSXMemberExpression';\n\tobject: JSXMemberExpressionObject;\n\tproperty: JSXIdentifier;\n}) & Span;";

impl<'a> Serialize for JSXExpressionContainer<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXExpressionContainer")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXExpressionContainer = ({\n\ttype: 'JSXExpressionContainer';\n\texpression: JSXExpression;\n}) & Span;";

impl<'a> Serialize for JSXExpression<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXExpression = JSXEmptyExpression | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl Serialize for JSXEmptyExpression {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXEmptyExpression")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type JSXEmptyExpression = ({\n\ttype: 'JSXEmptyExpression';\n}) & Span;";

impl<'a> Serialize for JSXAttributeItem<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            JSXAttributeItem::Attribute(x) => Serialize::serialize(x, serializer),
            JSXAttributeItem::SpreadAttribute(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type JSXAttributeItem = JSXAttribute | JSXSpreadAttribute;";

impl<'a> Serialize for JSXAttribute<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXAttribute")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXAttribute = ({\n\ttype: 'JSXAttribute';\n\tname: JSXAttributeName;\n\tvalue: (JSXAttributeValue) | null;\n}) & Span;";

impl<'a> Serialize for JSXSpreadAttribute<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXSpreadAttribute")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("argument", &self.argument)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXSpreadAttribute = ({\n\ttype: 'JSXSpreadAttribute';\n\targument: Expression;\n}) & Span;";

impl<'a> Serialize for JSXAttributeName<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            JSXAttributeName::Identifier(x) => Serialize::serialize(x, serializer),
            JSXAttributeName::NamespacedName(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type JSXAttributeName = JSXIdentifier | JSXNamespacedName;";

impl<'a> Serialize for JSXAttributeValue<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            JSXAttributeValue::StringLiteral(x) => Serialize::serialize(x, serializer),
            JSXAttributeValue::ExpressionContainer(x) => Serialize::serialize(x, serializer),
            JSXAttributeValue::Element(x) => Serialize::serialize(x, serializer),
            JSXAttributeValue::Fragment(x) => Serialize::serialize(x, serializer),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXAttributeValue = StringLiteral | JSXExpressionContainer | JSXElement | JSXFragment;";

impl<'a> Serialize for JSXIdentifier<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXIdentifier")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type JSXIdentifier = ({\n\ttype: 'JSXIdentifier';\n\tname: string;\n}) & Span;";

impl<'a> Serialize for JSXChild<'a> {
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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXChild = JSXText | JSXElement | JSXFragment | JSXExpressionContainer | JSXSpreadChild;";

impl<'a> Serialize for JSXSpreadChild<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXSpreadChild")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("expression", &self.expression)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXSpreadChild = ({\n\ttype: 'JSXSpreadChild';\n\texpression: Expression;\n}) & Span;";

impl<'a> Serialize for JSXText<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "JSXText")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type JSXText = ({\n\ttype: 'JSXText';\n\tvalue: string;\n}) & Span;";
