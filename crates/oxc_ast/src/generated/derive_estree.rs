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
const TS_APPEND_CONTENT: &'static str = "export type BooleanLiteral = ({\n    type: \"BooleanLiteral\";\n    value: boolean;\n}) & Span;";

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
    "export type NullLiteral = ({\n    type: \"NullLiteral\";\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type NumericLiteral = ({\n    type: \"NumericLiteral\";\n    value: number;\n    raw: string;\n}) & Span;";

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
    "export type BigIntLiteral = ({\n    type: \"BigIntLiteral\";\n    raw: string;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type RegExpLiteral = ({\n    type: \"RegExpLiteral\";\n    value: EmptyObject;\n    regex: RegExp;\n}) & Span;";

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
    "export type RegExp = ({\n    pattern: RegExpPattern;\n    flags: RegExpFlags;\n});";

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
    "export type StringLiteral = ({\n    type: \"StringLiteral\";\n    value: string;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type Program = ({\n    type: \"Program\";\n    sourceType: SourceType;\n    hashbang: (Hashbang) | null;\n    directives: Array<Directive>;\n    body: Array<Statement>;\n}) & Span;";

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
    "export type IdentifierName = ({\n    type: \"Identifier\";\n    name: string;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type IdentifierReference = ({\n    type: \"Identifier\";\n    name: string;\n}) & Span;";

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
    "export type BindingIdentifier = ({\n    type: \"Identifier\";\n    name: string;\n}) & Span;";

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
    "export type LabelIdentifier = ({\n    type: \"Identifier\";\n    name: string;\n}) & Span;";

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
    "export type ThisExpression = ({\n    type: \"ThisExpression\";\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ArrayExpression = ({\n    type: \"ArrayExpression\";\n    elements: Array<SpreadElement | Expression | null>;\n}) & Span;";

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

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ArrayExpressionElement = SpreadElement | Elision | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

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
const TS_APPEND_CONTENT: &'static str = "export type ObjectExpression = ({\n    type: \"ObjectExpression\";\n    properties: Array<ObjectPropertyKind>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ObjectProperty = ({\n    type: \"ObjectProperty\";\n    kind: PropertyKind;\n    key: PropertyKey;\n    value: Expression;\n    init: (Expression) | null;\n    method: boolean;\n    shorthand: boolean;\n    computed: boolean;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type PropertyKind = \"init\" | \"get\" | \"set\";";

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
const TS_APPEND_CONTENT: &'static str = "export type TemplateLiteral = ({\n    type: \"TemplateLiteral\";\n    quasis: Array<TemplateElement>;\n    expressions: Array<Expression>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TaggedTemplateExpression = ({\n    type: \"TaggedTemplateExpression\";\n    tag: Expression;\n    quasi: TemplateLiteral;\n    typeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TemplateElement = ({\n    type: \"TemplateElement\";\n    tail: boolean;\n    value: TemplateElementValue;\n}) & Span;";

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
    "export type TemplateElementValue = ({\n    raw: string;\n    cooked: (string) | null;\n});";

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
const TS_APPEND_CONTENT: &'static str = "export type ComputedMemberExpression = ({\n    type: \"ComputedMemberExpression\";\n    object: Expression;\n    expression: Expression;\n    optional: boolean;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type StaticMemberExpression = ({\n    type: \"StaticMemberExpression\";\n    object: Expression;\n    property: IdentifierName;\n    optional: boolean;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type PrivateFieldExpression = ({\n    type: \"PrivateFieldExpression\";\n    object: Expression;\n    field: PrivateIdentifier;\n    optional: boolean;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type CallExpression = ({\n    type: \"CallExpression\";\n    callee: Expression;\n    typeParameters: (TSTypeParameterInstantiation) | null;\n    arguments: Array<Argument>;\n    optional: boolean;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type NewExpression = ({\n    type: \"NewExpression\";\n    callee: Expression;\n    arguments: Array<Argument>;\n    typeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type MetaProperty = ({\n    type: \"MetaProperty\";\n    meta: IdentifierName;\n    property: IdentifierName;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type SpreadElement = ({\n    type: \"SpreadElement\";\n    argument: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type UpdateExpression = ({\n    type: \"UpdateExpression\";\n    operator: UpdateOperator;\n    prefix: boolean;\n    argument: SimpleAssignmentTarget;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type UnaryExpression = ({\n    type: \"UnaryExpression\";\n    operator: UnaryOperator;\n    argument: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type BinaryExpression = ({\n    type: \"BinaryExpression\";\n    left: Expression;\n    operator: BinaryOperator;\n    right: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type PrivateInExpression = ({\n    type: \"PrivateInExpression\";\n    left: PrivateIdentifier;\n    operator: BinaryOperator;\n    right: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type LogicalExpression = ({\n    type: \"LogicalExpression\";\n    left: Expression;\n    operator: LogicalOperator;\n    right: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ConditionalExpression = ({\n    type: \"ConditionalExpression\";\n    test: Expression;\n    consequent: Expression;\n    alternate: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type AssignmentExpression = ({\n    type: \"AssignmentExpression\";\n    operator: AssignmentOperator;\n    left: AssignmentTarget;\n    right: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ArrayAssignmentTarget = ({\n    type: \"ArrayAssignmentTarget\";\n    elements: Array<AssignmentTargetMaybeDefault | AssignmentTargetRest | null>;\n}) & Span;";

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ObjectAssignmentTarget = ({\n    type: \"ObjectAssignmentTarget\";\n    properties: Array<AssignmentTargetProperty | AssignmentTargetRest>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type AssignmentTargetRest = ({\n    type: \"RestElement\";\n    argument: AssignmentTarget;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type AssignmentTargetWithDefault = ({\n    type: \"AssignmentTargetWithDefault\";\n    binding: AssignmentTarget;\n    init: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type AssignmentTargetPropertyIdentifier = ({\n    type: \"AssignmentTargetPropertyIdentifier\";\n    binding: IdentifierReference;\n    init: (Expression) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type AssignmentTargetPropertyProperty = ({\n    type: \"AssignmentTargetPropertyProperty\";\n    name: PropertyKey;\n    binding: AssignmentTargetMaybeDefault;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type SequenceExpression = ({\n    type: \"SequenceExpression\";\n    expressions: Array<Expression>;\n}) & Span;";

impl Serialize for Super {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Super")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Super = ({\n    type: \"Super\";\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type AwaitExpression = ({\n    type: \"AwaitExpression\";\n    argument: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ChainExpression = ({\n    type: \"ChainExpression\";\n    expression: ChainElement;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ParenthesizedExpression = ({\n    type: \"ParenthesizedExpression\";\n    expression: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type Directive = ({\n    type: \"Directive\";\n    expression: StringLiteral;\n    directive: string;\n}) & Span;";

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
    "export type Hashbang = ({\n    type: \"Hashbang\";\n    value: string;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type BlockStatement = ({\n    type: \"BlockStatement\";\n    body: Array<Statement>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type VariableDeclaration = ({\n    type: \"VariableDeclaration\";\n    kind: VariableDeclarationKind;\n    declarations: Array<VariableDeclarator>;\n    declare: boolean;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type VariableDeclarationKind = \"var\" | \"const\" | \"let\" | \"using\" | \"await using\";";

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
const TS_APPEND_CONTENT: &'static str = "export type VariableDeclarator = ({\n    type: \"VariableDeclarator\";\n    id: BindingPattern;\n    init: (Expression) | null;\n    definite: boolean;\n}) & Span;";

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
    "export type EmptyStatement = ({\n    type: \"EmptyStatement\";\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ExpressionStatement = ({\n    type: \"ExpressionStatement\";\n    expression: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type IfStatement = ({\n    type: \"IfStatement\";\n    test: Expression;\n    consequent: Statement;\n    alternate: (Statement) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type DoWhileStatement = ({\n    type: \"DoWhileStatement\";\n    body: Statement;\n    test: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type WhileStatement = ({\n    type: \"WhileStatement\";\n    test: Expression;\n    body: Statement;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ForStatement = ({\n    type: \"ForStatement\";\n    init: (ForStatementInit) | null;\n    test: (Expression) | null;\n    update: (Expression) | null;\n    body: Statement;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ForInStatement = ({\n    type: \"ForInStatement\";\n    left: ForStatementLeft;\n    right: Expression;\n    body: Statement;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ForOfStatement = ({\n    type: \"ForOfStatement\";\n    await: boolean;\n    left: ForStatementLeft;\n    right: Expression;\n    body: Statement;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ContinueStatement = ({\n    type: \"ContinueStatement\";\n    label: (LabelIdentifier) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type BreakStatement = ({\n    type: \"BreakStatement\";\n    label: (LabelIdentifier) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ReturnStatement = ({\n    type: \"ReturnStatement\";\n    argument: (Expression) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type WithStatement = ({\n    type: \"WithStatement\";\n    object: Expression;\n    body: Statement;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type SwitchStatement = ({\n    type: \"SwitchStatement\";\n    discriminant: Expression;\n    cases: Array<SwitchCase>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type SwitchCase = ({\n    type: \"SwitchCase\";\n    test: (Expression) | null;\n    consequent: Array<Statement>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type LabeledStatement = ({\n    type: \"LabeledStatement\";\n    label: LabelIdentifier;\n    body: Statement;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ThrowStatement = ({\n    type: \"ThrowStatement\";\n    argument: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TryStatement = ({\n    type: \"TryStatement\";\n    block: BlockStatement;\n    handler: (CatchClause) | null;\n    finalizer: (BlockStatement) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type CatchClause = ({\n    type: \"CatchClause\";\n    param: (CatchParameter) | null;\n    body: BlockStatement;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type CatchParameter = ({\n    type: \"CatchParameter\";\n    pattern: BindingPattern;\n}) & Span;";

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
    "export type DebuggerStatement = ({\n    type: \"DebuggerStatement\";\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type BindingPattern = ({\n    typeAnnotation: (TSTypeAnnotation) | null;\n    optional: boolean;\n}) & (BindingIdentifier | ObjectPattern | ArrayPattern | AssignmentPattern);";

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
const TS_APPEND_CONTENT: &'static str = "export type AssignmentPattern = ({\n    type: \"AssignmentPattern\";\n    left: BindingPattern;\n    right: Expression;\n}) & Span;";

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ObjectPattern = ({\n    type: \"ObjectPattern\";\n    properties: Array<BindingProperty | BindingRestElement>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type BindingProperty = ({\n    type: \"BindingProperty\";\n    key: PropertyKey;\n    value: BindingPattern;\n    shorthand: boolean;\n    computed: boolean;\n}) & Span;";

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ArrayPattern = ({\n    type: \"ArrayPattern\";\n    elements: Array<BindingPattern | BindingRestElement | null>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type BindingRestElement = ({\n    type: \"RestElement\";\n    argument: BindingPattern;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type Function = ({\n    type: FunctionType;\n    id: (BindingIdentifier) | null;\n    generator: boolean;\n    async: boolean;\n    declare: boolean;\n    typeParameters: (TSTypeParameterDeclaration) | null;\n    thisParam: (TSThisParameter) | null;\n    params: FormalParameters;\n    returnType: (TSTypeAnnotation) | null;\n    body: (FunctionBody) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type FunctionType = \"FunctionDeclaration\" | \"FunctionExpression\" | \"TSDeclareFunction\" | \"TSEmptyBodyFunctionExpression\";";

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type FormalParameters = ({\n    type: \"FormalParameters\";\n    kind: FormalParameterKind;\n    items: Array<FormalParameter | FormalParameterRest>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type FormalParameter = ({\n    type: \"FormalParameter\";\n    decorators: Array<Decorator>;\n    pattern: BindingPattern;\n    accessibility: (TSAccessibility) | null;\n    readonly: boolean;\n    override: boolean;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type FormalParameterKind = \"FormalParameter\" | \"UniqueFormalParameters\" | \"ArrowFormalParameters\" | \"Signature\";";

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
const TS_APPEND_CONTENT: &'static str = "export type FunctionBody = ({\n    type: \"FunctionBody\";\n    directives: Array<Directive>;\n    statements: Array<Statement>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ArrowFunctionExpression = ({\n    type: \"ArrowFunctionExpression\";\n    expression: boolean;\n    async: boolean;\n    typeParameters: (TSTypeParameterDeclaration) | null;\n    params: FormalParameters;\n    returnType: (TSTypeAnnotation) | null;\n    body: FunctionBody;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type YieldExpression = ({\n    type: \"YieldExpression\";\n    delegate: boolean;\n    argument: (Expression) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type Class = ({\n    type: ClassType;\n    decorators: Array<Decorator>;\n    id: (BindingIdentifier) | null;\n    typeParameters: (TSTypeParameterDeclaration) | null;\n    superClass: (Expression) | null;\n    superTypeParameters: (TSTypeParameterInstantiation) | null;\n    implements: (Array<TSClassImplements>) | null;\n    body: ClassBody;\n    abstract: boolean;\n    declare: boolean;\n}) & Span;";

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
    "export type ClassType = \"ClassDeclaration\" | \"ClassExpression\";";

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
const TS_APPEND_CONTENT: &'static str = "export type ClassBody = ({\n    type: \"ClassBody\";\n    body: Array<ClassElement>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type MethodDefinition = ({\n    type: MethodDefinitionType;\n    decorators: Array<Decorator>;\n    key: PropertyKey;\n    value: Function;\n    kind: MethodDefinitionKind;\n    computed: boolean;\n    static: boolean;\n    override: boolean;\n    optional: boolean;\n    accessibility: (TSAccessibility) | null;\n}) & Span;";

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
    "export type MethodDefinitionType = \"MethodDefinition\" | \"TSAbstractMethodDefinition\";";

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
const TS_APPEND_CONTENT: &'static str = "export type PropertyDefinition = ({\n    type: PropertyDefinitionType;\n    decorators: Array<Decorator>;\n    key: PropertyKey;\n    value: (Expression) | null;\n    computed: boolean;\n    static: boolean;\n    declare: boolean;\n    override: boolean;\n    optional: boolean;\n    definite: boolean;\n    readonly: boolean;\n    typeAnnotation: (TSTypeAnnotation) | null;\n    accessibility: (TSAccessibility) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type PropertyDefinitionType = \"PropertyDefinition\" | \"TSAbstractPropertyDefinition\";";

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
    "export type MethodDefinitionKind = \"constructor\" | \"method\" | \"get\" | \"set\";";

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
const TS_APPEND_CONTENT: &'static str = "export type PrivateIdentifier = ({\n    type: \"PrivateIdentifier\";\n    name: string;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type StaticBlock = ({\n    type: \"StaticBlock\";\n    body: Array<Statement>;\n}) & Span;";

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
    "export type AccessorPropertyType = \"AccessorProperty\" | \"TSAbstractAccessorProperty\";";

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
const TS_APPEND_CONTENT: &'static str = "export type AccessorProperty = ({\n    type: AccessorPropertyType;\n    decorators: Array<Decorator>;\n    key: PropertyKey;\n    value: (Expression) | null;\n    computed: boolean;\n    static: boolean;\n    definite: boolean;\n    typeAnnotation: (TSTypeAnnotation) | null;\n    accessibility: (TSAccessibility) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ImportExpression = ({\n    type: \"ImportExpression\";\n    source: Expression;\n    arguments: Array<Expression>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ImportDeclaration = ({\n    type: \"ImportDeclaration\";\n    specifiers: (Array<ImportDeclarationSpecifier>) | null;\n    source: StringLiteral;\n    withClause: (WithClause) | null;\n    importKind: ImportOrExportKind;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ImportSpecifier = ({\n    type: \"ImportSpecifier\";\n    imported: ModuleExportName;\n    local: BindingIdentifier;\n    importKind: ImportOrExportKind;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ImportDefaultSpecifier = ({\n    type: \"ImportDefaultSpecifier\";\n    local: BindingIdentifier;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ImportNamespaceSpecifier = ({\n    type: \"ImportNamespaceSpecifier\";\n    local: BindingIdentifier;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type WithClause = ({\n    type: \"WithClause\";\n    attributesKeyword: IdentifierName;\n    withEntries: Array<ImportAttribute>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ImportAttribute = ({\n    type: \"ImportAttribute\";\n    key: ImportAttributeKey;\n    value: StringLiteral;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ExportNamedDeclaration = ({\n    type: \"ExportNamedDeclaration\";\n    declaration: (Declaration) | null;\n    specifiers: Array<ExportSpecifier>;\n    source: (StringLiteral) | null;\n    exportKind: ImportOrExportKind;\n    withClause: (WithClause) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ExportDefaultDeclaration = ({\n    type: \"ExportDefaultDeclaration\";\n    declaration: ExportDefaultDeclarationKind;\n    exported: ModuleExportName;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ExportAllDeclaration = ({\n    type: \"ExportAllDeclaration\";\n    exported: (ModuleExportName) | null;\n    source: StringLiteral;\n    withClause: (WithClause) | null;\n    exportKind: ImportOrExportKind;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ExportSpecifier = ({\n    type: \"ExportSpecifier\";\n    local: ModuleExportName;\n    exported: ModuleExportName;\n    exportKind: ImportOrExportKind;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSThisParameter = ({\n    type: \"TSThisParameter\";\n    thisSpan: Span;\n    typeAnnotation: (TSTypeAnnotation) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSEnumDeclaration = ({\n    type: \"TSEnumDeclaration\";\n    id: BindingIdentifier;\n    members: Array<TSEnumMember>;\n    const: boolean;\n    declare: boolean;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSEnumMember = ({\n    type: \"TSEnumMember\";\n    id: TSEnumMemberName;\n    initializer: (Expression) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSTypeAnnotation = ({\n    type: \"TSTypeAnnotation\";\n    typeAnnotation: TSType;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSLiteralType = ({\n    type: \"TSLiteralType\";\n    literal: TSLiteral;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSConditionalType = ({\n    type: \"TSConditionalType\";\n    checkType: TSType;\n    extendsType: TSType;\n    trueType: TSType;\n    falseType: TSType;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSUnionType = ({\n    type: \"TSUnionType\";\n    types: Array<TSType>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSIntersectionType = ({\n    type: \"TSIntersectionType\";\n    types: Array<TSType>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSParenthesizedType = ({\n    type: \"TSParenthesizedType\";\n    typeAnnotation: TSType;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSTypeOperator = ({\n    type: \"TSTypeOperator\";\n    operator: TSTypeOperatorOperator;\n    typeAnnotation: TSType;\n}) & Span;";

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
    "export type TSTypeOperatorOperator = \"keyof\" | \"unique\" | \"readonly\";";

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
const TS_APPEND_CONTENT: &'static str = "export type TSArrayType = ({\n    type: \"TSArrayType\";\n    elementType: TSType;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSIndexedAccessType = ({\n    type: \"TSIndexedAccessType\";\n    objectType: TSType;\n    indexType: TSType;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSTupleType = ({\n    type: \"TSTupleType\";\n    elementTypes: Array<TSTupleElement>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSNamedTupleMember = ({\n    type: \"TSNamedTupleMember\";\n    elementType: TSTupleElement;\n    label: IdentifierName;\n    optional: boolean;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSOptionalType = ({\n    type: \"TSOptionalType\";\n    typeAnnotation: TSType;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSRestType = ({\n    type: \"TSRestType\";\n    typeAnnotation: TSType;\n}) & Span;";

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
    "export type TSAnyKeyword = ({\n    type: \"TSAnyKeyword\";\n}) & Span;";

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
    "export type TSStringKeyword = ({\n    type: \"TSStringKeyword\";\n}) & Span;";

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
    "export type TSBooleanKeyword = ({\n    type: \"TSBooleanKeyword\";\n}) & Span;";

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
    "export type TSNumberKeyword = ({\n    type: \"TSNumberKeyword\";\n}) & Span;";

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
    "export type TSNeverKeyword = ({\n    type: \"TSNeverKeyword\";\n}) & Span;";

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
    "export type TSIntrinsicKeyword = ({\n    type: \"TSIntrinsicKeyword\";\n}) & Span;";

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
    "export type TSUnknownKeyword = ({\n    type: \"TSUnknownKeyword\";\n}) & Span;";

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
    "export type TSNullKeyword = ({\n    type: \"TSNullKeyword\";\n}) & Span;";

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
    "export type TSUndefinedKeyword = ({\n    type: \"TSUndefinedKeyword\";\n}) & Span;";

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
    "export type TSVoidKeyword = ({\n    type: \"TSVoidKeyword\";\n}) & Span;";

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
    "export type TSSymbolKeyword = ({\n    type: \"TSSymbolKeyword\";\n}) & Span;";

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
    "export type TSThisType = ({\n    type: \"TSThisType\";\n}) & Span;";

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
    "export type TSObjectKeyword = ({\n    type: \"TSObjectKeyword\";\n}) & Span;";

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
    "export type TSBigIntKeyword = ({\n    type: \"TSBigIntKeyword\";\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSTypeReference = ({\n    type: \"TSTypeReference\";\n    typeName: TSTypeName;\n    typeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSQualifiedName = ({\n    type: \"TSQualifiedName\";\n    left: TSTypeName;\n    right: IdentifierName;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSTypeParameterInstantiation = ({\n    type: \"TSTypeParameterInstantiation\";\n    params: Array<TSType>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSTypeParameter = ({\n    type: \"TSTypeParameter\";\n    name: BindingIdentifier;\n    constraint: (TSType) | null;\n    default: (TSType) | null;\n    in: boolean;\n    out: boolean;\n    const: boolean;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSTypeParameterDeclaration = ({\n    type: \"TSTypeParameterDeclaration\";\n    params: Array<TSTypeParameter>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSTypeAliasDeclaration = ({\n    type: \"TSTypeAliasDeclaration\";\n    id: BindingIdentifier;\n    typeParameters: (TSTypeParameterDeclaration) | null;\n    typeAnnotation: TSType;\n    declare: boolean;\n}) & Span;";

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
    "export type TSAccessibility = \"private\" | \"protected\" | \"public\";";

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
const TS_APPEND_CONTENT: &'static str = "export type TSClassImplements = ({\n    type: \"TSClassImplements\";\n    expression: TSTypeName;\n    typeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSInterfaceDeclaration = ({\n    type: \"TSInterfaceDeclaration\";\n    id: BindingIdentifier;\n    extends: (Array<TSInterfaceHeritage>) | null;\n    typeParameters: (TSTypeParameterDeclaration) | null;\n    body: TSInterfaceBody;\n    declare: boolean;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSInterfaceBody = ({\n    type: \"TSInterfaceBody\";\n    body: Array<TSSignature>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSPropertySignature = ({\n    type: \"TSPropertySignature\";\n    computed: boolean;\n    optional: boolean;\n    readonly: boolean;\n    key: PropertyKey;\n    typeAnnotation: (TSTypeAnnotation) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSIndexSignature = ({\n    type: \"TSIndexSignature\";\n    parameters: Array<TSIndexSignatureName>;\n    typeAnnotation: TSTypeAnnotation;\n    readonly: boolean;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSCallSignatureDeclaration = ({\n    type: \"TSCallSignatureDeclaration\";\n    typeParameters: (TSTypeParameterDeclaration) | null;\n    thisParam: (TSThisParameter) | null;\n    params: FormalParameters;\n    returnType: (TSTypeAnnotation) | null;\n}) & Span;";

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
    "export type TSMethodSignatureKind = \"method\" | \"get\" | \"set\";";

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
const TS_APPEND_CONTENT: &'static str = "export type TSMethodSignature = ({\n    type: \"TSMethodSignature\";\n    key: PropertyKey;\n    computed: boolean;\n    optional: boolean;\n    kind: TSMethodSignatureKind;\n    typeParameters: (TSTypeParameterDeclaration) | null;\n    thisParam: (TSThisParameter) | null;\n    params: FormalParameters;\n    returnType: (TSTypeAnnotation) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSConstructSignatureDeclaration = ({\n    type: \"TSConstructSignatureDeclaration\";\n    typeParameters: (TSTypeParameterDeclaration) | null;\n    params: FormalParameters;\n    returnType: (TSTypeAnnotation) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSIndexSignatureName = ({\n    type: \"Identifier\";\n    name: string;\n    typeAnnotation: TSTypeAnnotation;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSInterfaceHeritage = ({\n    type: \"TSInterfaceHeritage\";\n    expression: Expression;\n    typeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSTypePredicate = ({\n    type: \"TSTypePredicate\";\n    parameterName: TSTypePredicateName;\n    asserts: boolean;\n    typeAnnotation: (TSTypeAnnotation) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSModuleDeclaration = ({\n    type: \"TSModuleDeclaration\";\n    id: TSModuleDeclarationName;\n    body: (TSModuleDeclarationBody) | null;\n    kind: TSModuleDeclarationKind;\n    declare: boolean;\n}) & Span;";

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
    "export type TSModuleDeclarationKind = \"global\" | \"module\" | \"namespace\";";

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
const TS_APPEND_CONTENT: &'static str = "export type TSModuleBlock = ({\n    type: \"TSModuleBlock\";\n    body: Array<Statement>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSTypeLiteral = ({\n    type: \"TSTypeLiteral\";\n    members: Array<TSSignature>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSInferType = ({\n    type: \"TSInferType\";\n    typeParameter: TSTypeParameter;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSTypeQuery = ({\n    type: \"TSTypeQuery\";\n    exprName: TSTypeQueryExprName;\n    typeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSImportType = ({\n    type: \"TSImportType\";\n    isTypeOf: boolean;\n    parameter: TSType;\n    qualifier: (TSTypeName) | null;\n    attributes: (TSImportAttributes) | null;\n    typeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSImportAttributes = ({\n    type: \"TSImportAttributes\";\n    attributesKeyword: IdentifierName;\n    elements: Array<TSImportAttribute>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSImportAttribute = ({\n    type: \"TSImportAttribute\";\n    name: TSImportAttributeName;\n    value: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSFunctionType = ({\n    type: \"TSFunctionType\";\n    typeParameters: (TSTypeParameterDeclaration) | null;\n    thisParam: (TSThisParameter) | null;\n    params: FormalParameters;\n    returnType: TSTypeAnnotation;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSConstructorType = ({\n    type: \"TSConstructorType\";\n    abstract: boolean;\n    typeParameters: (TSTypeParameterDeclaration) | null;\n    params: FormalParameters;\n    returnType: TSTypeAnnotation;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSMappedType = ({\n    type: \"TSMappedType\";\n    typeParameter: TSTypeParameter;\n    nameType: (TSType) | null;\n    typeAnnotation: (TSType) | null;\n    optional: TSMappedTypeModifierOperator;\n    readonly: TSMappedTypeModifierOperator;\n}) & Span;";

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
    "export type TSMappedTypeModifierOperator = \"true\" | \"+\" | \"-\" | \"none\";";

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
const TS_APPEND_CONTENT: &'static str = "export type TSTemplateLiteralType = ({\n    type: \"TSTemplateLiteralType\";\n    quasis: Array<TemplateElement>;\n    types: Array<TSType>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSAsExpression = ({\n    type: \"TSAsExpression\";\n    expression: Expression;\n    typeAnnotation: TSType;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSSatisfiesExpression = ({\n    type: \"TSSatisfiesExpression\";\n    expression: Expression;\n    typeAnnotation: TSType;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSTypeAssertion = ({\n    type: \"TSTypeAssertion\";\n    expression: Expression;\n    typeAnnotation: TSType;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSImportEqualsDeclaration = ({\n    type: \"TSImportEqualsDeclaration\";\n    id: BindingIdentifier;\n    moduleReference: TSModuleReference;\n    importKind: ImportOrExportKind;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSExternalModuleReference = ({\n    type: \"TSExternalModuleReference\";\n    expression: StringLiteral;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSNonNullExpression = ({\n    type: \"TSNonNullExpression\";\n    expression: Expression;\n}) & Span;";

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
    "export type Decorator = ({\n    type: \"Decorator\";\n    expression: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSExportAssignment = ({\n    type: \"TSExportAssignment\";\n    expression: Expression;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSNamespaceExportDeclaration = ({\n    type: \"TSNamespaceExportDeclaration\";\n    id: IdentifierName;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type TSInstantiationExpression = ({\n    type: \"TSInstantiationExpression\";\n    expression: Expression;\n    typeParameters: TSTypeParameterInstantiation;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type ImportOrExportKind = \"value\" | \"type\";";

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
const TS_APPEND_CONTENT: &'static str = "export type JSDocNullableType = ({\n    type: \"JSDocNullableType\";\n    typeAnnotation: TSType;\n    postfix: boolean;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type JSDocNonNullableType = ({\n    type: \"JSDocNonNullableType\";\n    typeAnnotation: TSType;\n    postfix: boolean;\n}) & Span;";

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
    "export type JSDocUnknownType = ({\n    type: \"JSDocUnknownType\";\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type JSXElement = ({\n    type: \"JSXElement\";\n    openingElement: JSXOpeningElement;\n    closingElement: (JSXClosingElement) | null;\n    children: Array<JSXChild>;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type JSXOpeningElement = ({\n    type: \"JSXOpeningElement\";\n    selfClosing: boolean;\n    name: JSXElementName;\n    attributes: Array<JSXAttributeItem>;\n    typeParameters: (TSTypeParameterInstantiation) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type JSXClosingElement = ({\n    type: \"JSXClosingElement\";\n    name: JSXElementName;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type JSXFragment = ({\n    type: \"JSXFragment\";\n    openingFragment: JSXOpeningFragment;\n    closingFragment: JSXClosingFragment;\n    children: Array<JSXChild>;\n}) & Span;";

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
    "export type JSXOpeningFragment = ({\n    type: \"JSXOpeningFragment\";\n}) & Span;";

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
    "export type JSXClosingFragment = ({\n    type: \"JSXClosingFragment\";\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type JSXNamespacedName = ({\n    type: \"JSXNamespacedName\";\n    namespace: JSXIdentifier;\n    property: JSXIdentifier;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type JSXMemberExpression = ({\n    type: \"JSXMemberExpression\";\n    object: JSXMemberExpressionObject;\n    property: JSXIdentifier;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type JSXExpressionContainer = ({\n    type: \"JSXExpressionContainer\";\n    expression: JSXExpression;\n}) & Span;";

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
    "export type JSXEmptyExpression = ({\n    type: \"JSXEmptyExpression\";\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type JSXAttribute = ({\n    type: \"JSXAttribute\";\n    name: JSXAttributeName;\n    value: (JSXAttributeValue) | null;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type JSXSpreadAttribute = ({\n    type: \"JSXSpreadAttribute\";\n    argument: Expression;\n}) & Span;";

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
    "export type JSXIdentifier = ({\n    type: \"JSXIdentifier\";\n    name: string;\n}) & Span;";

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
const TS_APPEND_CONTENT: &'static str = "export type JSXSpreadChild = ({\n    type: \"JSXSpreadChild\";\n    expression: Expression;\n}) & Span;";

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
    "export type JSXText = ({\n    type: \"JSXText\";\n    value: string;\n}) & Span;";
