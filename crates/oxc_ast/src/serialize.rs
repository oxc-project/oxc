use cow_utils::CowUtils;
use num_bigint::BigInt;
use num_traits::Num;
use serde::{
    ser::{SerializeSeq, Serializer},
    Serialize,
};

use oxc_allocator::Box;
use oxc_span::{Atom, Span};
use oxc_syntax::number::BigintBase;

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
    raw: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bigint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    regex: Option<SerRegExpValue>,
}

impl From<&BooleanLiteral> for ESTreeLiteral<'_, bool> {
    fn from(lit: &BooleanLiteral) -> Self {
        let raw = if lit.span.is_unspanned() {
            None
        } else {
            Some(if lit.value { "true" } else { "false" })
        };

        Self { span: lit.span, value: lit.value, raw, bigint: None, regex: None }
    }
}

impl From<&NullLiteral> for ESTreeLiteral<'_, ()> {
    fn from(lit: &NullLiteral) -> Self {
        let raw = if lit.span.is_unspanned() { None } else { Some("null") };
        Self { span: lit.span, value: (), raw, bigint: None, regex: None }
    }
}

impl<'a> From<&'a NumericLiteral<'a>> for ESTreeLiteral<'a, f64> {
    fn from(lit: &'a NumericLiteral) -> Self {
        Self {
            span: lit.span,
            value: lit.value,
            raw: lit.raw.as_ref().map(Atom::as_str),
            bigint: None,
            regex: None,
        }
    }
}

impl<'a> From<&'a StringLiteral<'a>> for ESTreeLiteral<'a, &'a str> {
    fn from(lit: &'a StringLiteral) -> Self {
        Self {
            span: lit.span,
            value: &lit.value,
            raw: lit.raw.as_ref().map(Atom::as_str),
            bigint: None,
            regex: None,
        }
    }
}

impl<'a> From<&'a BigIntLiteral<'a>> for ESTreeLiteral<'a, ()> {
    fn from(lit: &'a BigIntLiteral) -> Self {
        let src = &lit.raw.strip_suffix('n').unwrap().cow_replace('_', "");

        let src = match lit.base {
            BigintBase::Decimal => src,
            BigintBase::Binary | BigintBase::Octal | BigintBase::Hex => &src[2..],
        };
        let radix = match lit.base {
            BigintBase::Decimal => 10,
            BigintBase::Binary => 2,
            BigintBase::Octal => 8,
            BigintBase::Hex => 16,
        };
        let bigint = BigInt::from_str_radix(src, radix).unwrap();

        Self {
            span: lit.span,
            // BigInts can't be serialized to JSON
            value: (),
            raw: Some(lit.raw.as_str()),
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
    fn from(lit: &'a RegExpLiteral) -> Self {
        Self {
            span: lit.span,
            raw: lit.raw.as_ref().map(Atom::as_str),
            value: match &lit.regex.pattern {
                RegExpPattern::Pattern(_) => Some(EmptyObject {}),
                _ => None,
            },
            bigint: None,
            regex: Some(SerRegExpValue {
                pattern: lit.regex.pattern.to_string(),
                flags: lit.regex.flags.to_string(),
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

impl Program<'_> {
    /// # Panics
    pub fn to_json(&self) -> String {
        let ser = self.serializer();
        String::from_utf8(ser.into_inner()).unwrap()
    }

    /// # Panics
    pub fn serializer(&self) -> serde_json::Serializer<Vec<u8>, EcmaFormatter> {
        let buf = Vec::new();
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
impl Serialize for FormalParameters<'_> {
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
            items: ElementsAndRest::new(&self.items, converted_rest.as_ref()),
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
    rest: Option<&'b R>,
}

impl<'b, E, R> ElementsAndRest<'b, E, R> {
    pub fn new(elements: &'b [E], rest: Option<&'b R>) -> Self {
        Self { elements, rest }
    }
}

impl<E: Serialize, R: Serialize> Serialize for ElementsAndRest<'_, E, R> {
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

pub struct OptionVecDefault<'a, 'b, T: Serialize>(pub &'a Option<oxc_allocator::Vec<'b, T>>);

impl<T: Serialize> Serialize for OptionVecDefault<'_, '_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(vec) = &self.0 {
            vec.serialize(serializer)
        } else {
            [false; 0].serialize(serializer)
        }
    }
}

/// Serialize `TSModuleBlock` to be ESTree compatible, with `body` and `directives` fields combined,
/// and directives output as `StringLiteral` expression statements
impl Serialize for TSModuleBlock<'_> {
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

impl Serialize for DirectivesAndStatements<'_, '_> {
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

impl Serialize for JSXElementName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Identifier(ident) => ident.serialize(serializer),
            Self::IdentifierReference(ident) => {
                JSXIdentifier { span: ident.span, name: ident.name }.serialize(serializer)
            }
            Self::NamespacedName(name) => name.serialize(serializer),
            Self::MemberExpression(expr) => expr.serialize(serializer),
            Self::ThisExpression(expr) => {
                JSXIdentifier { span: expr.span, name: "this".into() }.serialize(serializer)
            }
        }
    }
}

impl Serialize for JSXMemberExpressionObject<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::IdentifierReference(ident) => {
                JSXIdentifier { span: ident.span, name: ident.name }.serialize(serializer)
            }
            Self::MemberExpression(expr) => expr.serialize(serializer),
            Self::ThisExpression(expr) => {
                JSXIdentifier { span: expr.span, name: "this".into() }.serialize(serializer)
            }
        }
    }
}
