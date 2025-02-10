use std::io::Write;

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

/// Constant value that will be serialized as `null` in JSON.
pub(crate) const NULL: () = ();

impl Program<'_> {
    /// Serialize AST to JSON.
    //
    // Should not panic if everything is working correctly.
    // Serializing into a `Vec<u8>` should be infallible.
    #[expect(clippy::missing_panics_doc)]
    pub fn to_json(&self) -> String {
        let buf = Vec::new();
        let ser = self.to_json_into_writer(buf).unwrap();
        let buf = ser.into_inner();
        // SAFETY: `serde_json` outputs valid UTF-8.
        // `serde_json::to_string` also uses `from_utf8_unchecked`.
        // https://github.com/serde-rs/json/blob/1174c5f57db44c26460951b525c6ede50984b655/src/ser.rs#L2209-L2219
        unsafe { String::from_utf8_unchecked(buf) }
    }

    /// Serialize AST into a "black hole" writer.
    ///
    /// Only useful for testing, to make sure serialization completes successfully.
    /// Should be faster than [`Program::to_json`], as does not actually produce any output.
    ///
    /// # Errors
    /// Returns `Err` if serialization fails.
    #[doc(hidden)]
    pub fn test_to_json(&self) -> Result<(), serde_json::Error> {
        struct BlackHole;

        #[expect(clippy::inline_always)]
        impl Write for BlackHole {
            #[inline(always)]
            fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
                Ok(buf.len())
            }

            #[inline(always)]
            fn flush(&mut self) -> Result<(), std::io::Error> {
                Ok(())
            }
        }

        self.to_json_into_writer(BlackHole).map(|_| ())
    }

    /// Serialize AST into the provided writer.
    fn to_json_into_writer<W: Write>(
        &self,
        writer: W,
    ) -> Result<serde_json::Serializer<W, EcmaFormatter>, serde_json::Error> {
        let mut ser = serde_json::Serializer::with_formatter(writer, EcmaFormatter);
        self.serialize(&mut ser)?;
        Ok(ser)
    }
}

/// `serde_json` formatter which uses `ryu_js` to serialize `f64`.
pub struct EcmaFormatter;

impl serde_json::ser::Formatter for EcmaFormatter {
    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        use oxc_syntax::number::ToJsString;
        writer.write_all(value.to_js_string().as_bytes())
    }
}

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

// --------------------
// Various
// --------------------

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

/// Wrap an `Option<Vec<T>>` so that it's serialized as an empty array (`[]`) if the `Option` is `None`.
pub struct OptionVecDefault<'a, 'b, T: Serialize>(pub &'b Option<ArenaVec<'a, T>>);

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

/// Serializer for `ArrowFunctionExpression`'s `body` field.
///
/// Serializes as either an expression (if `expression` property is set),
/// or a `BlockStatement` (if it's not).
pub struct ArrowFunctionExpressionBody<'a>(pub &'a ArrowFunctionExpression<'a>);

impl Serialize for ArrowFunctionExpressionBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(expression) = self.0.get_expression() {
            expression.serialize(serializer)
        } else {
            self.0.body.serialize(serializer)
        }
    }
}

/// Serializer for `AssignmentTargetPropertyIdentifier`'s `init` field
/// (which is renamed to `value` in ESTree AST).
pub struct AssignmentTargetPropertyIdentifierValue<'a>(
    pub &'a AssignmentTargetPropertyIdentifier<'a>,
);

impl Serialize for AssignmentTargetPropertyIdentifierValue<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(init) = &self.0.init {
            let mut map = serializer.serialize_map(None)?;
            map.serialize_entry("type", "AssignmentPattern")?;
            map.serialize_entry("start", &self.0.span.start)?;
            map.serialize_entry("end", &self.0.span.end)?;
            map.serialize_entry("left", &self.0.binding)?;
            map.serialize_entry("right", init)?;
            map.end()
        } else {
            self.0.binding.serialize(serializer)
        }
    }
}

// --------------------
// JSX
// --------------------

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
