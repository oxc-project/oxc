use cow_utils::CowUtils;
use num_bigint::BigInt;
use num_traits::Num;
use serde::{
    ser::{SerializeMap, SerializeSeq, Serializer},
    Serialize,
};

use oxc_allocator::{Box as ArenaBox, Vec as ArenaVec};
use oxc_span::Span;
use oxc_syntax::number::BigintBase;

use crate::ast::*;

// --------------------
// Literals
// --------------------

/// Get `raw` field of `BooleanLiteral`.
pub fn boolean_literal_raw(lit: &BooleanLiteral) -> Option<&str> {
    if lit.span.is_unspanned() {
        None
    } else if lit.value {
        Some("true")
    } else {
        Some("false")
    }
}

/// Get `raw` field of `NullLiteral`.
pub fn null_literal_raw(lit: &NullLiteral) -> Option<&str> {
    if lit.span.is_unspanned() {
        None
    } else {
        Some("null")
    }
}

/// Get `bigint` field of `BigIntLiteral`.
pub fn bigint_literal_bigint(lit: &BigIntLiteral) -> String {
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

    BigInt::from_str_radix(src, radix).unwrap().to_string()
}

/// A placeholder for `RegExpLiteral`'s `value` field.
pub struct EmptyObject;

impl Serialize for EmptyObject {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let map = serializer.serialize_map(None)?;
        map.end()
    }
}

impl Serialize for RegExpFlags {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl Serialize for RegExpPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
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

/// Serialize `ArrayExpressionElement::Elision` variant as `null`.
impl Serialize for Elision {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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
        ElementsAndRest::new(&self.items, converted_rest.as_ref()).serialize(serializer)
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "RestElement", rename_all = "camelCase")]
struct SerFormalParameterRest<'a, 'b> {
    #[serde(flatten)]
    span: Span,
    argument: &'b BindingPatternKind<'a>,
    type_annotation: &'b Option<ArenaBox<'a, TSTypeAnnotation<'a>>>,
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

pub struct OptionVecDefault<'a, 'b, T: Serialize>(pub &'b Option<ArenaVec<'a, T>>);

impl<'a, 'b, T: Serialize> From<&'b Option<ArenaVec<'a, T>>> for OptionVecDefault<'a, 'b, T> {
    fn from(opt_vec: &'b Option<ArenaVec<'a, T>>) -> Self {
        Self(opt_vec)
    }
}

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
