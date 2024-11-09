use cow_utils::CowUtils;
use num_bigint::BigInt;
use num_traits::Num;
use oxc_allocator::Box;
use oxc_span::Span;
use oxc_syntax::number::BigintBase;
use serde::{
    ser::{SerializeSeq, Serializer},
    Serialize,
};

use crate::ast::{
    BigIntLiteral, BindingPatternKind, BooleanLiteral, Directive, Elision, FormalParameter,
    FormalParameterKind, FormalParameters, JSXElementName, JSXIdentifier,
    JSXMemberExpressionObject, NullLiteral, NumericLiteral, Program, RegExpFlags, RegExpLiteral,
    RegExpPattern, Statement, StringLiteral, TSModuleBlock, TSTypeAnnotation,
};

#[derive(Serialize)]
#[serde(tag = "type", rename = "Literal")]
pub struct ESTreeLiteral<'a, T> {
    #[serde(flatten)]
    span: Span,
    value: T,
    raw: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    bigint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    regex: Option<SerRegExpValue>,
}

impl<'a> From<&BooleanLiteral> for ESTreeLiteral<'a, bool> {
    fn from(value: &BooleanLiteral) -> Self {
        Self {
            span: value.span,
            value: value.value,
            raw: if value.value { "true" } else { "false" },
            bigint: None,
            regex: None,
        }
    }
}

impl<'a> From<&NullLiteral> for ESTreeLiteral<'a, ()> {
    fn from(value: &NullLiteral) -> Self {
        Self { span: value.span, value: (), raw: "null", bigint: None, regex: None }
    }
}

impl<'a> From<&'a NumericLiteral<'a>> for ESTreeLiteral<'a, f64> {
    fn from(value: &'a NumericLiteral) -> Self {
        Self { span: value.span, value: value.value, raw: value.raw, bigint: None, regex: None }
    }
}

impl<'a> From<&'a StringLiteral<'a>> for ESTreeLiteral<'a, &'a str> {
    fn from(value: &'a StringLiteral) -> Self {
        Self { span: value.span, value: &value.value, raw: value.raw, bigint: None, regex: None }
    }
}

impl<'a> From<&'a BigIntLiteral<'a>> for ESTreeLiteral<'a, ()> {
    fn from(value: &'a BigIntLiteral) -> Self {
        let src = &value.raw.strip_suffix('n').unwrap().cow_replace('_', "");

        let src = match value.base {
            BigintBase::Decimal => src,
            BigintBase::Binary | BigintBase::Octal | BigintBase::Hex => &src[2..],
        };
        let radix = match value.base {
            BigintBase::Decimal => 10,
            BigintBase::Binary => 2,
            BigintBase::Octal => 8,
            BigintBase::Hex => 16,
        };
        let bigint = BigInt::from_str_radix(src, radix).unwrap();

        Self {
            span: value.span,
            // BigInts can't be serialized to JSON
            value: (),
            raw: value.raw.as_str(),
            bigint: Some(bigint.to_string()),
            regex: None,
        }
    }
}
#[derive(Serialize)]
pub struct SerRegExpValue {
    pattern: String,
    flags: String,
}

/// A placeholder for regexp literals that can't be serialized to JSON
#[derive(Serialize)]
#[allow(clippy::empty_structs_with_brackets)]
pub struct EmptyObject {}

impl<'a> From<&'a RegExpLiteral<'a>> for ESTreeLiteral<'a, Option<EmptyObject>> {
    fn from(value: &'a RegExpLiteral) -> Self {
        Self {
            span: value.span,
            raw: value.raw,
            value: match &value.regex.pattern {
                RegExpPattern::Pattern(_) => Some(EmptyObject {}),
                _ => None,
            },
            bigint: None,
            regex: Some(SerRegExpValue {
                pattern: value.regex.pattern.to_string(),
                flags: value.regex.flags.to_string(),
            }),
        }
    }
}

pub struct EcmaFormatter;

/// Serialize f64 with `ryu_js`
impl serde_json::ser::Formatter for EcmaFormatter {
    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        use oxc_syntax::number::ToJsString;
        writer.write_all(value.to_js_string().as_bytes())
    }
}

impl<'a> Program<'a> {
    /// # Panics
    pub fn to_json(&self) -> String {
        let ser = self.serializer();
        String::from_utf8(ser.into_inner()).unwrap()
    }

    /// # Panics
    pub fn serializer(&self) -> serde_json::Serializer<std::vec::Vec<u8>, EcmaFormatter> {
        let buf = std::vec::Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(buf, EcmaFormatter);
        self.serialize(&mut ser).unwrap();
        ser
    }
}

impl Serialize for RegExpFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Serialize `ArrayExpressionElement::Elision` variant as `null` in JSON
impl Serialize for Elision {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

/// Serialize `FormalParameters`, to be estree compatible, with `items` and `rest` fields combined
/// and `argument` field flattened.
impl<'a> Serialize for FormalParameters<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let converted_rest = self.rest.as_ref().map(|rest| SerFormalParameterRest {
            span: rest.span,
            argument: &rest.argument.kind,
            type_annotation: &rest.argument.type_annotation,
            optional: rest.argument.optional,
        });
        let converted = SerFormalParameters {
            span: self.span,
            kind: self.kind,
            items: ElementsAndRest::new(&self.items, &converted_rest),
        };
        converted.serialize(serializer)
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "FormalParameters")]
struct SerFormalParameters<'a, 'b> {
    #[serde(flatten)]
    span: Span,
    kind: FormalParameterKind,
    items: ElementsAndRest<'b, FormalParameter<'a>, SerFormalParameterRest<'a, 'b>>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "RestElement", rename_all = "camelCase")]
struct SerFormalParameterRest<'a, 'b> {
    #[serde(flatten)]
    span: Span,
    argument: &'b BindingPatternKind<'a>,
    type_annotation: &'b Option<Box<'a, TSTypeAnnotation<'a>>>,
    optional: bool,
}

pub struct ElementsAndRest<'b, E, R> {
    elements: &'b [E],
    rest: &'b Option<R>,
}

impl<'b, E, R> ElementsAndRest<'b, E, R> {
    pub fn new(elements: &'b [E], rest: &'b Option<R>) -> Self {
        Self { elements, rest }
    }
}

impl<'b, E: Serialize, R: Serialize> Serialize for ElementsAndRest<'b, E, R> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(rest) = self.rest {
            let mut seq = serializer.serialize_seq(Some(self.elements.len() + 1))?;
            for element in self.elements {
                seq.serialize_element(element)?;
            }
            seq.serialize_element(rest)?;
            seq.end()
        } else {
            self.elements.serialize(serializer)
        }
    }
}

/// Serialize `TSModuleBlock` to be ESTree compatible, with `body` and `directives` fields combined,
/// and directives output as `StringLiteral` expression statements
impl<'a> Serialize for TSModuleBlock<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let converted = SerTSModuleBlock {
            span: self.span,
            body: DirectivesAndStatements { directives: &self.directives, body: &self.body },
        };
        converted.serialize(serializer)
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "TSModuleBlock")]
struct SerTSModuleBlock<'a, 'b> {
    #[serde(flatten)]
    span: Span,
    body: DirectivesAndStatements<'a, 'b>,
}

struct DirectivesAndStatements<'a, 'b> {
    directives: &'b [Directive<'a>],
    body: &'b [Statement<'a>],
}

impl<'a, 'b> Serialize for DirectivesAndStatements<'a, 'b> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.directives.len() + self.body.len()))?;
        for directive in self.directives {
            seq.serialize_element(&DirectiveAsStatement {
                span: directive.span,
                expression: &directive.expression,
            })?;
        }
        for stmt in self.body {
            seq.serialize_element(stmt)?;
        }
        seq.end()
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "ExpressionStatement")]
struct DirectiveAsStatement<'a, 'b> {
    #[serde(flatten)]
    span: Span,
    expression: &'b StringLiteral<'a>,
}

impl<'a> Serialize for JSXElementName<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Identifier(ident) => ident.serialize(serializer),
            Self::IdentifierReference(ident) => {
                JSXIdentifier { span: ident.span, name: ident.name.clone() }.serialize(serializer)
            }
            Self::NamespacedName(name) => name.serialize(serializer),
            Self::MemberExpression(expr) => expr.serialize(serializer),
            Self::ThisExpression(expr) => {
                JSXIdentifier { span: expr.span, name: "this".into() }.serialize(serializer)
            }
        }
    }
}

impl<'a> Serialize for JSXMemberExpressionObject<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::IdentifierReference(ident) => {
                JSXIdentifier { span: ident.span, name: ident.name.clone() }.serialize(serializer)
            }
            Self::MemberExpression(expr) => expr.serialize(serializer),
            Self::ThisExpression(expr) => {
                JSXIdentifier { span: expr.span, name: "this".into() }.serialize(serializer)
            }
        }
    }
}
