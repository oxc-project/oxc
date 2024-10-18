// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/estree.rs`

#[allow(unused_imports)]
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            RegExpPattern::Raw(ref x) => Serialize::serialize(x, serializer),
            RegExpPattern::Invalid(ref x) => Serialize::serialize(x, serializer),
            RegExpPattern::Pattern(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type RegExpPattern = string | string | Pattern;";

impl Serialize for EmptyObject {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.end()
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type EmptyObject = ({\n});";

impl<'a> Serialize for StringLiteral<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Expression::BooleanLiteral(ref x) => Serialize::serialize(x, serializer),
            Expression::NullLiteral(ref x) => Serialize::serialize(x, serializer),
            Expression::NumericLiteral(ref x) => Serialize::serialize(x, serializer),
            Expression::BigIntLiteral(ref x) => Serialize::serialize(x, serializer),
            Expression::RegExpLiteral(ref x) => Serialize::serialize(x, serializer),
            Expression::StringLiteral(ref x) => Serialize::serialize(x, serializer),
            Expression::TemplateLiteral(ref x) => Serialize::serialize(x, serializer),
            Expression::Identifier(ref x) => Serialize::serialize(x, serializer),
            Expression::MetaProperty(ref x) => Serialize::serialize(x, serializer),
            Expression::Super(ref x) => Serialize::serialize(x, serializer),
            Expression::ArrayExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::ArrowFunctionExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::AssignmentExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::AwaitExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::BinaryExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::CallExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::ChainExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::ClassExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::ConditionalExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::FunctionExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::ImportExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::LogicalExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::NewExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::ObjectExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::ParenthesizedExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::SequenceExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::TaggedTemplateExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::ThisExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::UnaryExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::UpdateExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::YieldExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::PrivateInExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::JSXElement(ref x) => Serialize::serialize(x, serializer),
            Expression::JSXFragment(ref x) => Serialize::serialize(x, serializer),
            Expression::TSAsExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::TSSatisfiesExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::TSTypeAssertion(ref x) => Serialize::serialize(x, serializer),
            Expression::TSNonNullExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::TSInstantiationExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::ComputedMemberExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::StaticMemberExpression(ref x) => Serialize::serialize(x, serializer),
            Expression::PrivateFieldExpression(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Expression = BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for IdentifierName<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ArrayExpressionElement::SpreadElement(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::Elision(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::BooleanLiteral(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::NullLiteral(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::NumericLiteral(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::BigIntLiteral(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::RegExpLiteral(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::StringLiteral(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::TemplateLiteral(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::Identifier(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::MetaProperty(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::Super(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::ArrayExpression(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::ArrowFunctionExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::AssignmentExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::AwaitExpression(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::BinaryExpression(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::CallExpression(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::ChainExpression(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::ClassExpression(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::ConditionalExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::FunctionExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::ImportExpression(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::LogicalExpression(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::NewExpression(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::ObjectExpression(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::ParenthesizedExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::SequenceExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::TaggedTemplateExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::ThisExpression(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::UnaryExpression(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::UpdateExpression(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::YieldExpression(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::PrivateInExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::JSXElement(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::JSXFragment(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::TSAsExpression(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::TSSatisfiesExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::TSTypeAssertion(ref x) => Serialize::serialize(x, serializer),
            ArrayExpressionElement::TSNonNullExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::TSInstantiationExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::ComputedMemberExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::StaticMemberExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ArrayExpressionElement::PrivateFieldExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ArrayExpressionElement = SpreadElement | Elision | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for ObjectExpression<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ObjectPropertyKind::ObjectProperty(ref x) => Serialize::serialize(x, serializer),
            ObjectPropertyKind::SpreadProperty(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type ObjectPropertyKind = ObjectProperty | SpreadElement;";

impl<'a> Serialize for ObjectProperty<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            PropertyKey::StaticIdentifier(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::PrivateIdentifier(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::BooleanLiteral(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::NullLiteral(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::NumericLiteral(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::BigIntLiteral(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::RegExpLiteral(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::StringLiteral(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::TemplateLiteral(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::Identifier(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::MetaProperty(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::Super(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::ArrayExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::ArrowFunctionExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::AssignmentExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::AwaitExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::BinaryExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::CallExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::ChainExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::ClassExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::ConditionalExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::FunctionExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::ImportExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::LogicalExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::NewExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::ObjectExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::ParenthesizedExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::SequenceExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::TaggedTemplateExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::ThisExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::UnaryExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::UpdateExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::YieldExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::PrivateInExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::JSXElement(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::JSXFragment(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::TSAsExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::TSSatisfiesExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::TSTypeAssertion(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::TSNonNullExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::TSInstantiationExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::ComputedMemberExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::StaticMemberExpression(ref x) => Serialize::serialize(x, serializer),
            PropertyKey::PrivateFieldExpression(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type PropertyKey = IdentifierName | PrivateIdentifier | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl Serialize for PropertyKind {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            MemberExpression::ComputedMemberExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            MemberExpression::StaticMemberExpression(ref x) => Serialize::serialize(x, serializer),
            MemberExpression::PrivateFieldExpression(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type MemberExpression = ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for ComputedMemberExpression<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Argument::SpreadElement(ref x) => Serialize::serialize(x, serializer),
            Argument::BooleanLiteral(ref x) => Serialize::serialize(x, serializer),
            Argument::NullLiteral(ref x) => Serialize::serialize(x, serializer),
            Argument::NumericLiteral(ref x) => Serialize::serialize(x, serializer),
            Argument::BigIntLiteral(ref x) => Serialize::serialize(x, serializer),
            Argument::RegExpLiteral(ref x) => Serialize::serialize(x, serializer),
            Argument::StringLiteral(ref x) => Serialize::serialize(x, serializer),
            Argument::TemplateLiteral(ref x) => Serialize::serialize(x, serializer),
            Argument::Identifier(ref x) => Serialize::serialize(x, serializer),
            Argument::MetaProperty(ref x) => Serialize::serialize(x, serializer),
            Argument::Super(ref x) => Serialize::serialize(x, serializer),
            Argument::ArrayExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::ArrowFunctionExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::AssignmentExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::AwaitExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::BinaryExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::CallExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::ChainExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::ClassExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::ConditionalExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::FunctionExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::ImportExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::LogicalExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::NewExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::ObjectExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::ParenthesizedExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::SequenceExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::TaggedTemplateExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::ThisExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::UnaryExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::UpdateExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::YieldExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::PrivateInExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::JSXElement(ref x) => Serialize::serialize(x, serializer),
            Argument::JSXFragment(ref x) => Serialize::serialize(x, serializer),
            Argument::TSAsExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::TSSatisfiesExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::TSTypeAssertion(ref x) => Serialize::serialize(x, serializer),
            Argument::TSNonNullExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::TSInstantiationExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::ComputedMemberExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::StaticMemberExpression(ref x) => Serialize::serialize(x, serializer),
            Argument::PrivateFieldExpression(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Argument = SpreadElement | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for UpdateExpression<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            AssignmentTarget::AssignmentTargetIdentifier(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTarget::TSAsExpression(ref x) => Serialize::serialize(x, serializer),
            AssignmentTarget::TSSatisfiesExpression(ref x) => Serialize::serialize(x, serializer),
            AssignmentTarget::TSNonNullExpression(ref x) => Serialize::serialize(x, serializer),
            AssignmentTarget::TSTypeAssertion(ref x) => Serialize::serialize(x, serializer),
            AssignmentTarget::TSInstantiationExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTarget::ComputedMemberExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTarget::StaticMemberExpression(ref x) => Serialize::serialize(x, serializer),
            AssignmentTarget::PrivateFieldExpression(ref x) => Serialize::serialize(x, serializer),
            AssignmentTarget::ArrayAssignmentTarget(ref x) => Serialize::serialize(x, serializer),
            AssignmentTarget::ObjectAssignmentTarget(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type AssignmentTarget = IdentifierReference | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression | ArrayAssignmentTarget | ObjectAssignmentTarget;";

impl<'a> Serialize for SimpleAssignmentTarget<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(ref x) => {
                Serialize::serialize(x, serializer)
            }
            SimpleAssignmentTarget::TSAsExpression(ref x) => Serialize::serialize(x, serializer),
            SimpleAssignmentTarget::TSSatisfiesExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            SimpleAssignmentTarget::TSNonNullExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            SimpleAssignmentTarget::TSTypeAssertion(ref x) => Serialize::serialize(x, serializer),
            SimpleAssignmentTarget::TSInstantiationExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            SimpleAssignmentTarget::ComputedMemberExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            SimpleAssignmentTarget::StaticMemberExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            SimpleAssignmentTarget::PrivateFieldExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type SimpleAssignmentTarget = IdentifierReference | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for AssignmentTargetPattern<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            AssignmentTargetPattern::ArrayAssignmentTarget(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(ref x) => {
                Serialize::serialize(x, serializer)
            }
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type AssignmentTargetPattern = ArrayAssignmentTarget | ObjectAssignmentTarget;";

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ArrayAssignmentTarget = ({\n\telements: Array<AssignmentTargetMaybeDefault | AssignmentTargetRest | null>;\n}) & Span;";

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ObjectAssignmentTarget = ({\n\tproperties: Array<AssignmentTargetProperty | AssignmentTargetRest>;\n}) & Span;";

impl<'a> Serialize for AssignmentTargetRest<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::TSAsExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::TSSatisfiesExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::TSNonNullExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::TSTypeAssertion(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::TSInstantiationExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::ComputedMemberExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::StaticMemberExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::PrivateFieldExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::ArrayAssignmentTarget(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(ref x) => {
                Serialize::serialize(x, serializer)
            }
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type AssignmentTargetMaybeDefault = AssignmentTargetWithDefault | IdentifierReference | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression | ArrayAssignmentTarget | ObjectAssignmentTarget;";

impl<'a> Serialize for AssignmentTargetWithDefault<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ref x) => {
                Serialize::serialize(x, serializer)
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(ref x) => {
                Serialize::serialize(x, serializer)
            }
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type AssignmentTargetProperty = AssignmentTargetPropertyIdentifier | AssignmentTargetPropertyProperty;";

impl<'a> Serialize for AssignmentTargetPropertyIdentifier<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Super")?;
        self.span.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Super = ({\n\ttype: 'Super';\n}) & Span;";

impl<'a> Serialize for AwaitExpression<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ChainElement::CallExpression(ref x) => Serialize::serialize(x, serializer),
            ChainElement::ComputedMemberExpression(ref x) => Serialize::serialize(x, serializer),
            ChainElement::StaticMemberExpression(ref x) => Serialize::serialize(x, serializer),
            ChainElement::PrivateFieldExpression(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ChainElement = CallExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for ParenthesizedExpression<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Statement::BlockStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::BreakStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::ContinueStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::DebuggerStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::DoWhileStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::EmptyStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::ExpressionStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::ForInStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::ForOfStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::ForStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::IfStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::LabeledStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::ReturnStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::SwitchStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::ThrowStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::TryStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::WhileStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::WithStatement(ref x) => Serialize::serialize(x, serializer),
            Statement::VariableDeclaration(ref x) => Serialize::serialize(x, serializer),
            Statement::FunctionDeclaration(ref x) => Serialize::serialize(x, serializer),
            Statement::ClassDeclaration(ref x) => Serialize::serialize(x, serializer),
            Statement::TSTypeAliasDeclaration(ref x) => Serialize::serialize(x, serializer),
            Statement::TSInterfaceDeclaration(ref x) => Serialize::serialize(x, serializer),
            Statement::TSEnumDeclaration(ref x) => Serialize::serialize(x, serializer),
            Statement::TSModuleDeclaration(ref x) => Serialize::serialize(x, serializer),
            Statement::TSImportEqualsDeclaration(ref x) => Serialize::serialize(x, serializer),
            Statement::ImportDeclaration(ref x) => Serialize::serialize(x, serializer),
            Statement::ExportAllDeclaration(ref x) => Serialize::serialize(x, serializer),
            Statement::ExportDefaultDeclaration(ref x) => Serialize::serialize(x, serializer),
            Statement::ExportNamedDeclaration(ref x) => Serialize::serialize(x, serializer),
            Statement::TSExportAssignment(ref x) => Serialize::serialize(x, serializer),
            Statement::TSNamespaceExportDeclaration(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Statement = BlockStatement | BreakStatement | ContinueStatement | DebuggerStatement | DoWhileStatement | EmptyStatement | ExpressionStatement | ForInStatement | ForOfStatement | ForStatement | IfStatement | LabeledStatement | ReturnStatement | SwitchStatement | ThrowStatement | TryStatement | WhileStatement | WithStatement | VariableDeclaration | Function | Class | TSTypeAliasDeclaration | TSInterfaceDeclaration | TSEnumDeclaration | TSModuleDeclaration | TSImportEqualsDeclaration | ImportDeclaration | ExportAllDeclaration | ExportDefaultDeclaration | ExportNamedDeclaration | TSExportAssignment | TSNamespaceExportDeclaration;";

impl<'a> Serialize for Directive<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Declaration::VariableDeclaration(ref x) => Serialize::serialize(x, serializer),
            Declaration::FunctionDeclaration(ref x) => Serialize::serialize(x, serializer),
            Declaration::ClassDeclaration(ref x) => Serialize::serialize(x, serializer),
            Declaration::TSTypeAliasDeclaration(ref x) => Serialize::serialize(x, serializer),
            Declaration::TSInterfaceDeclaration(ref x) => Serialize::serialize(x, serializer),
            Declaration::TSEnumDeclaration(ref x) => Serialize::serialize(x, serializer),
            Declaration::TSModuleDeclaration(ref x) => Serialize::serialize(x, serializer),
            Declaration::TSImportEqualsDeclaration(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type Declaration = VariableDeclaration | Function | Class | TSTypeAliasDeclaration | TSInterfaceDeclaration | TSEnumDeclaration | TSModuleDeclaration | TSImportEqualsDeclaration;";

impl<'a> Serialize for VariableDeclaration<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ForStatementInit::VariableDeclaration(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::BooleanLiteral(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::NullLiteral(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::NumericLiteral(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::BigIntLiteral(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::RegExpLiteral(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::StringLiteral(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::TemplateLiteral(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::Identifier(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::MetaProperty(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::Super(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::ArrayExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::ArrowFunctionExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::AssignmentExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::AwaitExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::BinaryExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::CallExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::ChainExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::ClassExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::ConditionalExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::FunctionExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::ImportExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::LogicalExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::NewExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::ObjectExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::ParenthesizedExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::SequenceExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::TaggedTemplateExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ForStatementInit::ThisExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::UnaryExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::UpdateExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::YieldExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::PrivateInExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::JSXElement(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::JSXFragment(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::TSAsExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::TSSatisfiesExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::TSTypeAssertion(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::TSNonNullExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::TSInstantiationExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ForStatementInit::ComputedMemberExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ForStatementInit::StaticMemberExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementInit::PrivateFieldExpression(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ForStatementInit = VariableDeclaration | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for ForInStatement<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ForStatementLeft::VariableDeclaration(ref x) => Serialize::serialize(x, serializer),
            ForStatementLeft::AssignmentTargetIdentifier(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ForStatementLeft::TSAsExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementLeft::TSSatisfiesExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementLeft::TSNonNullExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementLeft::TSTypeAssertion(ref x) => Serialize::serialize(x, serializer),
            ForStatementLeft::TSInstantiationExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ForStatementLeft::ComputedMemberExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ForStatementLeft::StaticMemberExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementLeft::PrivateFieldExpression(ref x) => Serialize::serialize(x, serializer),
            ForStatementLeft::ArrayAssignmentTarget(ref x) => Serialize::serialize(x, serializer),
            ForStatementLeft::ObjectAssignmentTarget(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ForStatementLeft = VariableDeclaration | IdentifierReference | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression | ArrayAssignmentTarget | ObjectAssignmentTarget;";

impl<'a> Serialize for ForOfStatement<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            BindingPatternKind::BindingIdentifier(ref x) => Serialize::serialize(x, serializer),
            BindingPatternKind::ObjectPattern(ref x) => Serialize::serialize(x, serializer),
            BindingPatternKind::ArrayPattern(ref x) => Serialize::serialize(x, serializer),
            BindingPatternKind::AssignmentPattern(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type BindingPatternKind = BindingIdentifier | ObjectPattern | ArrayPattern | AssignmentPattern;";

impl<'a> Serialize for AssignmentPattern<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
const TS_APPEND_CONTENT: &'static str = "export type ObjectPattern = ({\n\tproperties: Array<BindingProperty | BindingRestElement>;\n}) & Span;";

impl<'a> Serialize for BindingProperty<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
const TS_APPEND_CONTENT: &'static str = "export type ArrayPattern = ({\n\telements: Array<BindingPattern | BindingRestElement | null>;\n}) & Span;";

impl<'a> Serialize for BindingRestElement<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
const TS_APPEND_CONTENT: &'static str = "export type FormalParameters = ({\n\tkind: FormalParameterKind;\n\titems: Array<FormalParameter | FormalParameterRest>;\n}) & Span;";

impl<'a> Serialize for FormalParameter<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ClassElement::StaticBlock(ref x) => Serialize::serialize(x, serializer),
            ClassElement::MethodDefinition(ref x) => Serialize::serialize(x, serializer),
            ClassElement::PropertyDefinition(ref x) => Serialize::serialize(x, serializer),
            ClassElement::AccessorProperty(ref x) => Serialize::serialize(x, serializer),
            ClassElement::TSIndexSignature(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ClassElement = StaticBlock | MethodDefinition | PropertyDefinition | AccessorProperty | TSIndexSignature;";

impl<'a> Serialize for MethodDefinition<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ModuleDeclaration::ImportDeclaration(ref x) => Serialize::serialize(x, serializer),
            ModuleDeclaration::ExportAllDeclaration(ref x) => Serialize::serialize(x, serializer),
            ModuleDeclaration::ExportDefaultDeclaration(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ModuleDeclaration::ExportNamedDeclaration(ref x) => Serialize::serialize(x, serializer),
            ModuleDeclaration::TSExportAssignment(ref x) => Serialize::serialize(x, serializer),
            ModuleDeclaration::TSNamespaceExportDeclaration(ref x) => {
                Serialize::serialize(x, serializer)
            }
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ModuleDeclaration = ImportDeclaration | ExportAllDeclaration | ExportDefaultDeclaration | ExportNamedDeclaration | TSExportAssignment | TSNamespaceExportDeclaration;";

impl Serialize for AccessorPropertyType {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ImportDeclarationSpecifier::ImportSpecifier(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(ref x) => {
                Serialize::serialize(x, serializer)
            }
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ImportDeclarationSpecifier = ImportSpecifier | ImportDefaultSpecifier | ImportNamespaceSpecifier;";

impl<'a> Serialize for ImportSpecifier<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ImportAttributeKey::Identifier(ref x) => Serialize::serialize(x, serializer),
            ImportAttributeKey::StringLiteral(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type ImportAttributeKey = IdentifierName | StringLiteral;";

impl<'a> Serialize for ExportNamedDeclaration<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ExportDefaultDeclarationKind::FunctionDeclaration(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::ClassDeclaration(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::BooleanLiteral(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::NullLiteral(ref x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::NumericLiteral(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::BigIntLiteral(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::RegExpLiteral(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::StringLiteral(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::TemplateLiteral(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::Identifier(ref x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::MetaProperty(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::Super(ref x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::ArrayExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::ArrowFunctionExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::AssignmentExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::AwaitExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::BinaryExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::CallExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::ChainExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::ClassExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::ConditionalExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::FunctionExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::ImportExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::LogicalExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::NewExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::ObjectExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::ParenthesizedExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::SequenceExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::TaggedTemplateExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::ThisExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::UnaryExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::UpdateExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::YieldExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::PrivateInExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::JSXElement(ref x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::JSXFragment(ref x) => Serialize::serialize(x, serializer),
            ExportDefaultDeclarationKind::TSAsExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::TSSatisfiesExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::TSTypeAssertion(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::TSNonNullExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::TSInstantiationExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::ComputedMemberExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::StaticMemberExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            ExportDefaultDeclarationKind::PrivateFieldExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type ExportDefaultDeclarationKind = Function | Class | TSInterfaceDeclaration | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for ModuleExportName<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ModuleExportName::IdentifierName(ref x) => Serialize::serialize(x, serializer),
            ModuleExportName::IdentifierReference(ref x) => Serialize::serialize(x, serializer),
            ModuleExportName::StringLiteral(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type ModuleExportName = IdentifierName | IdentifierReference | StringLiteral;";

impl<'a> Serialize for TSThisParameter<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            TSEnumMemberName::StaticIdentifier(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::StaticStringLiteral(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::StaticTemplateLiteral(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::StaticNumericLiteral(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::BooleanLiteral(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::NullLiteral(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::NumericLiteral(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::BigIntLiteral(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::RegExpLiteral(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::StringLiteral(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::TemplateLiteral(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::Identifier(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::MetaProperty(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::Super(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ArrayExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ArrowFunctionExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::AssignmentExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::AwaitExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::BinaryExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::CallExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ChainExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ClassExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ConditionalExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::FunctionExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ImportExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::LogicalExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::NewExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ObjectExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::ParenthesizedExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::SequenceExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::TaggedTemplateExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            TSEnumMemberName::ThisExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::UnaryExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::UpdateExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::YieldExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::PrivateInExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::JSXElement(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::JSXFragment(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::TSAsExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::TSSatisfiesExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::TSTypeAssertion(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::TSNonNullExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::TSInstantiationExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            TSEnumMemberName::ComputedMemberExpression(ref x) => {
                Serialize::serialize(x, serializer)
            }
            TSEnumMemberName::StaticMemberExpression(ref x) => Serialize::serialize(x, serializer),
            TSEnumMemberName::PrivateFieldExpression(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSEnumMemberName = IdentifierName | StringLiteral | TemplateLiteral | NumericLiteral | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl<'a> Serialize for TSTypeAnnotation<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            TSLiteral::BooleanLiteral(ref x) => Serialize::serialize(x, serializer),
            TSLiteral::NullLiteral(ref x) => Serialize::serialize(x, serializer),
            TSLiteral::NumericLiteral(ref x) => Serialize::serialize(x, serializer),
            TSLiteral::BigIntLiteral(ref x) => Serialize::serialize(x, serializer),
            TSLiteral::RegExpLiteral(ref x) => Serialize::serialize(x, serializer),
            TSLiteral::StringLiteral(ref x) => Serialize::serialize(x, serializer),
            TSLiteral::TemplateLiteral(ref x) => Serialize::serialize(x, serializer),
            TSLiteral::UnaryExpression(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSLiteral = BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | UnaryExpression;";

impl<'a> Serialize for TSType<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            TSType::TSAnyKeyword(ref x) => Serialize::serialize(x, serializer),
            TSType::TSBigIntKeyword(ref x) => Serialize::serialize(x, serializer),
            TSType::TSBooleanKeyword(ref x) => Serialize::serialize(x, serializer),
            TSType::TSIntrinsicKeyword(ref x) => Serialize::serialize(x, serializer),
            TSType::TSNeverKeyword(ref x) => Serialize::serialize(x, serializer),
            TSType::TSNullKeyword(ref x) => Serialize::serialize(x, serializer),
            TSType::TSNumberKeyword(ref x) => Serialize::serialize(x, serializer),
            TSType::TSObjectKeyword(ref x) => Serialize::serialize(x, serializer),
            TSType::TSStringKeyword(ref x) => Serialize::serialize(x, serializer),
            TSType::TSSymbolKeyword(ref x) => Serialize::serialize(x, serializer),
            TSType::TSUndefinedKeyword(ref x) => Serialize::serialize(x, serializer),
            TSType::TSUnknownKeyword(ref x) => Serialize::serialize(x, serializer),
            TSType::TSVoidKeyword(ref x) => Serialize::serialize(x, serializer),
            TSType::TSArrayType(ref x) => Serialize::serialize(x, serializer),
            TSType::TSConditionalType(ref x) => Serialize::serialize(x, serializer),
            TSType::TSConstructorType(ref x) => Serialize::serialize(x, serializer),
            TSType::TSFunctionType(ref x) => Serialize::serialize(x, serializer),
            TSType::TSImportType(ref x) => Serialize::serialize(x, serializer),
            TSType::TSIndexedAccessType(ref x) => Serialize::serialize(x, serializer),
            TSType::TSInferType(ref x) => Serialize::serialize(x, serializer),
            TSType::TSIntersectionType(ref x) => Serialize::serialize(x, serializer),
            TSType::TSLiteralType(ref x) => Serialize::serialize(x, serializer),
            TSType::TSMappedType(ref x) => Serialize::serialize(x, serializer),
            TSType::TSNamedTupleMember(ref x) => Serialize::serialize(x, serializer),
            TSType::TSQualifiedName(ref x) => Serialize::serialize(x, serializer),
            TSType::TSTemplateLiteralType(ref x) => Serialize::serialize(x, serializer),
            TSType::TSThisType(ref x) => Serialize::serialize(x, serializer),
            TSType::TSTupleType(ref x) => Serialize::serialize(x, serializer),
            TSType::TSTypeLiteral(ref x) => Serialize::serialize(x, serializer),
            TSType::TSTypeOperatorType(ref x) => Serialize::serialize(x, serializer),
            TSType::TSTypePredicate(ref x) => Serialize::serialize(x, serializer),
            TSType::TSTypeQuery(ref x) => Serialize::serialize(x, serializer),
            TSType::TSTypeReference(ref x) => Serialize::serialize(x, serializer),
            TSType::TSUnionType(ref x) => Serialize::serialize(x, serializer),
            TSType::TSParenthesizedType(ref x) => Serialize::serialize(x, serializer),
            TSType::JSDocNullableType(ref x) => Serialize::serialize(x, serializer),
            TSType::JSDocNonNullableType(ref x) => Serialize::serialize(x, serializer),
            TSType::JSDocUnknownType(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSType = TSAnyKeyword | TSBigIntKeyword | TSBooleanKeyword | TSIntrinsicKeyword | TSNeverKeyword | TSNullKeyword | TSNumberKeyword | TSObjectKeyword | TSStringKeyword | TSSymbolKeyword | TSUndefinedKeyword | TSUnknownKeyword | TSVoidKeyword | TSArrayType | TSConditionalType | TSConstructorType | TSFunctionType | TSImportType | TSIndexedAccessType | TSInferType | TSIntersectionType | TSLiteralType | TSMappedType | TSNamedTupleMember | TSQualifiedName | TSTemplateLiteralType | TSThisType | TSTupleType | TSTypeLiteral | TSTypeOperator | TSTypePredicate | TSTypeQuery | TSTypeReference | TSUnionType | TSParenthesizedType | JSDocNullableType | JSDocNonNullableType | JSDocUnknownType;";

impl<'a> Serialize for TSConditionalType<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            TSTupleElement::TSOptionalType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSRestType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSAnyKeyword(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSBigIntKeyword(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSBooleanKeyword(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSIntrinsicKeyword(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSNeverKeyword(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSNullKeyword(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSNumberKeyword(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSObjectKeyword(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSStringKeyword(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSSymbolKeyword(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSUndefinedKeyword(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSUnknownKeyword(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSVoidKeyword(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSArrayType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSConditionalType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSConstructorType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSFunctionType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSImportType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSIndexedAccessType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSInferType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSIntersectionType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSLiteralType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSMappedType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSNamedTupleMember(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSQualifiedName(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSTemplateLiteralType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSThisType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSTupleType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSTypeLiteral(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSTypeOperatorType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSTypePredicate(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSTypeQuery(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSTypeReference(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSUnionType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::TSParenthesizedType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::JSDocNullableType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::JSDocNonNullableType(ref x) => Serialize::serialize(x, serializer),
            TSTupleElement::JSDocUnknownType(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSTupleElement = TSOptionalType | TSRestType | TSAnyKeyword | TSBigIntKeyword | TSBooleanKeyword | TSIntrinsicKeyword | TSNeverKeyword | TSNullKeyword | TSNumberKeyword | TSObjectKeyword | TSStringKeyword | TSSymbolKeyword | TSUndefinedKeyword | TSUnknownKeyword | TSVoidKeyword | TSArrayType | TSConditionalType | TSConstructorType | TSFunctionType | TSImportType | TSIndexedAccessType | TSInferType | TSIntersectionType | TSLiteralType | TSMappedType | TSNamedTupleMember | TSQualifiedName | TSTemplateLiteralType | TSThisType | TSTupleType | TSTypeLiteral | TSTypeOperator | TSTypePredicate | TSTypeQuery | TSTypeReference | TSUnionType | TSParenthesizedType | JSDocNullableType | JSDocNonNullableType | JSDocUnknownType;";

impl Serialize for TSAnyKeyword {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            TSTypeName::IdentifierReference(ref x) => Serialize::serialize(x, serializer),
            TSTypeName::QualifiedName(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSTypeName = IdentifierReference | TSQualifiedName;";

impl<'a> Serialize for TSQualifiedName<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            TSSignature::TSIndexSignature(ref x) => Serialize::serialize(x, serializer),
            TSSignature::TSPropertySignature(ref x) => Serialize::serialize(x, serializer),
            TSSignature::TSCallSignatureDeclaration(ref x) => Serialize::serialize(x, serializer),
            TSSignature::TSConstructSignatureDeclaration(ref x) => {
                Serialize::serialize(x, serializer)
            }
            TSSignature::TSMethodSignature(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSSignature = TSIndexSignature | TSPropertySignature | TSCallSignatureDeclaration | TSConstructSignatureDeclaration | TSMethodSignature;";

impl<'a> Serialize for TSIndexSignature<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            TSTypePredicateName::Identifier(ref x) => Serialize::serialize(x, serializer),
            TSTypePredicateName::This(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSTypePredicateName = IdentifierName | TSThisType;";

impl<'a> Serialize for TSModuleDeclaration<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            TSModuleDeclarationName::Identifier(ref x) => Serialize::serialize(x, serializer),
            TSModuleDeclarationName::StringLiteral(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSModuleDeclarationName = BindingIdentifier | StringLiteral;";

impl<'a> Serialize for TSModuleDeclarationBody<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            TSModuleDeclarationBody::TSModuleDeclaration(ref x) => {
                Serialize::serialize(x, serializer)
            }
            TSModuleDeclarationBody::TSModuleBlock(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSModuleDeclarationBody = TSModuleDeclaration | TSModuleBlock;";

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSModuleBlock = ({\n\tbody: Array<Statement>;\n}) & Span;";

impl<'a> Serialize for TSTypeLiteral<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            TSTypeQueryExprName::TSImportType(ref x) => Serialize::serialize(x, serializer),
            TSTypeQueryExprName::IdentifierReference(ref x) => Serialize::serialize(x, serializer),
            TSTypeQueryExprName::QualifiedName(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSTypeQueryExprName = TSImportType | IdentifierReference | TSQualifiedName;";

impl<'a> Serialize for TSImportType<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            TSImportAttributeName::Identifier(ref x) => Serialize::serialize(x, serializer),
            TSImportAttributeName::StringLiteral(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type TSImportAttributeName = IdentifierName | StringLiteral;";

impl<'a> Serialize for TSFunctionType<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            TSModuleReference::ExternalModuleReference(ref x) => {
                Serialize::serialize(x, serializer)
            }
            TSModuleReference::IdentifierReference(ref x) => Serialize::serialize(x, serializer),
            TSModuleReference::QualifiedName(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type TSModuleReference = TSExternalModuleReference | IdentifierReference | TSQualifiedName;";

impl<'a> Serialize for TSExternalModuleReference<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            JSXExpression::EmptyExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::BooleanLiteral(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::NullLiteral(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::NumericLiteral(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::BigIntLiteral(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::RegExpLiteral(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::StringLiteral(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::TemplateLiteral(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::Identifier(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::MetaProperty(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::Super(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::ArrayExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::ArrowFunctionExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::AssignmentExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::AwaitExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::BinaryExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::CallExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::ChainExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::ClassExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::ConditionalExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::FunctionExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::ImportExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::LogicalExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::NewExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::ObjectExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::ParenthesizedExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::SequenceExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::TaggedTemplateExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::ThisExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::UnaryExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::UpdateExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::YieldExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::PrivateInExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::JSXElement(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::JSXFragment(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::TSAsExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::TSSatisfiesExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::TSTypeAssertion(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::TSNonNullExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::TSInstantiationExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::ComputedMemberExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::StaticMemberExpression(ref x) => Serialize::serialize(x, serializer),
            JSXExpression::PrivateFieldExpression(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXExpression = JSXEmptyExpression | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | IdentifierReference | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | Class | ConditionalExpression | Function | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;";

impl Serialize for JSXEmptyExpression {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            JSXAttributeItem::Attribute(ref x) => Serialize::serialize(x, serializer),
            JSXAttributeItem::SpreadAttribute(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type JSXAttributeItem = JSXAttribute | JSXSpreadAttribute;";

impl<'a> Serialize for JSXAttribute<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            JSXAttributeName::Identifier(ref x) => Serialize::serialize(x, serializer),
            JSXAttributeName::NamespacedName(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str =
    "export type JSXAttributeName = JSXIdentifier | JSXNamespacedName;";

impl<'a> Serialize for JSXAttributeValue<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            JSXAttributeValue::StringLiteral(ref x) => Serialize::serialize(x, serializer),
            JSXAttributeValue::ExpressionContainer(ref x) => Serialize::serialize(x, serializer),
            JSXAttributeValue::Element(ref x) => Serialize::serialize(x, serializer),
            JSXAttributeValue::Fragment(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXAttributeValue = StringLiteral | JSXExpressionContainer | JSXElement | JSXFragment;";

impl<'a> Serialize for JSXIdentifier<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            JSXChild::Text(ref x) => Serialize::serialize(x, serializer),
            JSXChild::Element(ref x) => Serialize::serialize(x, serializer),
            JSXChild::Fragment(ref x) => Serialize::serialize(x, serializer),
            JSXChild::ExpressionContainer(ref x) => Serialize::serialize(x, serializer),
            JSXChild::Spread(ref x) => Serialize::serialize(x, serializer),
        }
    }
}
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = "export type JSXChild = JSXText | JSXElement | JSXFragment | JSXExpressionContainer | JSXSpreadChild;";

impl<'a> Serialize for JSXSpreadChild<'a> {
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    #[allow(clippy::match_same_arms, unused_mut)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
